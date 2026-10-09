/* QA-only ALSA boundary observer. Never reads audio buffers. ABI described in
 * V4. */
#define _GNU_SOURCE
#include <alsa/asoundlib.h>
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

#if !defined(__x86_64__) || __BYTE_ORDER__ != __ORDER_LITTLE_ENDIAN__
#error diagnostic ABI requires Linux x86-64 little endian
#endif
_Static_assert(ATOMIC_LLONG_LOCK_FREE == 2,
               "diagnostic counters must be lock free");
typedef _Atomic uint64_t Word;
typedef struct {
  Word count, negative, first_negative, epipes, first_epipe_ns;
  Word last_ns, max_gap_ns, max_duration_ns, max_frames;
} Calls;
typedef struct {
  Word handle, closed, params_calls, params_error, buffer, period, channels,
      rate;
  Calls avail, write;
  Word prepares, recovers, max_processing_ns;
} Pcm;
typedef struct {
  Word magic, version, pid, used, lost, ready, clock_failures, start_ns;
  Pcm pcm[64];
} Evidence;
_Static_assert(sizeof(Word) == 8 && sizeof(Pcm) == 29 * 8,
               "unexpected diagnostic ABI");
_Static_assert(sizeof(Evidence) == (8 + 64 * 29) * 8,
               "unexpected evidence size");
static Evidence *evidence;
static _Thread_local Pcm *cached;
static _Thread_local uint64_t cached_handle;
static _Thread_local uint64_t avail_end_ns;
static _Thread_local unsigned int depth;

static int (*next_open)(snd_pcm_t **, const char *, snd_pcm_stream_t, int);
static int (*next_close)(snd_pcm_t *);
static int (*next_params)(snd_pcm_t *, snd_pcm_hw_params_t *);
static int (*next_get_params)(snd_pcm_t *, snd_pcm_uframes_t *,
                              snd_pcm_uframes_t *);
static int (*next_channels)(const snd_pcm_hw_params_t *, unsigned int *);
static int (*next_rate)(const snd_pcm_hw_params_t *, unsigned int *, int *);
static snd_pcm_sframes_t (*next_avail)(snd_pcm_t *);
static snd_pcm_sframes_t (*next_write)(snd_pcm_t *, const void *,
                                       snd_pcm_uframes_t);
static int (*next_prepare)(snd_pcm_t *);
static int (*next_recover)(snd_pcm_t *, int, int);

static uint64_t now_ns(void) {
  struct timespec ts;
  if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
    atomic_fetch_add_explicit(&evidence->clock_failures, 1,
                              memory_order_relaxed);
    return 0;
  }
  return (uint64_t)ts.tv_sec * 1000000000u + (uint64_t)ts.tv_nsec;
}

static void maximum(Word *word, uint64_t value) {
  uint64_t old = atomic_load_explicit(word, memory_order_relaxed);
  for (int attempt = 0; value > old && attempt < 4; ++attempt) {
    if (atomic_compare_exchange_weak_explicit(
            word, &old, value, memory_order_relaxed, memory_order_relaxed))
      return;
  }
  if (value > old)
    atomic_fetch_add_explicit(&evidence->lost, 1, memory_order_relaxed);
}

static void require_owner(void) {
  if (atomic_load_explicit(&evidence->pid, memory_order_relaxed) !=
      (uint64_t)getpid())
    _exit(126);
}

static Pcm *find_pcm(snd_pcm_t *pcm) {
  require_owner();
  uint64_t handle = (uint64_t)(uintptr_t)pcm;
  if (cached && cached_handle == handle &&
      !atomic_load_explicit(&cached->closed, memory_order_relaxed))
    return cached;
  uint64_t used = atomic_load_explicit(&evidence->used, memory_order_acquire);
  for (uint64_t i = 0; i < used && i < 64; ++i) {
    Pcm *slot = &evidence->pcm[i];
    if (atomic_load_explicit(&slot->handle, memory_order_acquire) == handle &&
        !atomic_load_explicit(&slot->closed, memory_order_relaxed)) {
      cached = slot;
      cached_handle = handle;
      avail_end_ns = 0;
      return slot;
    }
  }
  _exit(126); /* never silently bypass the measured boundary */
}

#define RESOLVE(target, name, version)                                         \
  do {                                                                         \
    void *symbol = dlvsym(RTLD_NEXT, name, version);                           \
    _Static_assert(sizeof(target) == sizeof(symbol),                           \
                   "function pointer ABI mismatch");                           \
    if (!symbol)                                                               \
      _exit(125);                                                              \
    memcpy(&(target), &symbol, sizeof(symbol));                                \
  } while (0)

__attribute__((constructor)) static void initialize(void) {
  const char *path = getenv("RIOTBOX_ALSA_DIAGNOSTICS");
  if (!path || path[0] != '/')
    _exit(125);
  int fd = open(path, O_RDWR | O_NOFOLLOW | O_CLOEXEC);
  struct stat st;
  if (fd < 0 || fstat(fd, &st) || !S_ISREG(st.st_mode) ||
      st.st_uid != getuid() || st.st_nlink != 1 ||
      (st.st_mode & 0777) != 0600 || st.st_size != sizeof(Evidence))
    _exit(125);
  evidence =
      mmap(NULL, sizeof(Evidence), PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
  close(fd);
  if (evidence == MAP_FAILED)
    _exit(125);
  /* Prefault before any audio call; fixed mapping, not a realtime file write.
   */
  volatile unsigned char *bytes = (volatile unsigned char *)evidence;
  for (size_t i = 0; i < sizeof(Evidence); ++i)
    if (bytes[i])
      _exit(125);
  for (size_t i = 0; i < sizeof(Evidence); i += sizeof(Word))
    atomic_init((Word *)((unsigned char *)evidence + i), 0);
  for (size_t i = 0; i < sizeof(Evidence); i += 4096)
    bytes[i] = 0;
  bytes[sizeof(Evidence) - 1] = 0;
  RESOLVE(next_open, "snd_pcm_open", "ALSA_0.9");
  RESOLVE(next_close, "snd_pcm_close", "ALSA_0.9");
  RESOLVE(next_params, "snd_pcm_hw_params", "ALSA_0.9");
  RESOLVE(next_get_params, "snd_pcm_get_params", "ALSA_0.9");
  RESOLVE(next_channels, "snd_pcm_hw_params_get_channels", "ALSA_0.9.0rc4");
  RESOLVE(next_rate, "snd_pcm_hw_params_get_rate", "ALSA_0.9.0rc4");
  RESOLVE(next_avail, "snd_pcm_avail", "ALSA_0.9");
  RESOLVE(next_write, "snd_pcm_writei", "ALSA_0.9");
  RESOLVE(next_prepare, "snd_pcm_prepare", "ALSA_0.9");
  RESOLVE(next_recover, "snd_pcm_recover", "ALSA_0.9");
  atomic_store(&evidence->magic, UINT64_C(0x3141534c41584252));
  atomic_store(&evidence->version, 1);
  atomic_store(&evidence->pid, getpid());
  atomic_store(&evidence->start_ns, now_ns());
  atomic_store(&evidence->ready, 1);
}

int snd_pcm_open(snd_pcm_t **pcm, const char *name, snd_pcm_stream_t stream,
                 int mode) {
  int outer = depth++ == 0;
  if (outer)
    require_owner();
  int result = next_open(pcm, name, stream, mode), saved = errno;
  if (outer && result == 0) {
    if (stream != SND_PCM_STREAM_PLAYBACK)
      _exit(126);
    uint64_t index = atomic_fetch_add(&evidence->used, 1);
    if (index >= 64)
      _exit(126);
    atomic_store_explicit(&evidence->pcm[index].handle, (uintptr_t)*pcm,
                          memory_order_release);
  }
  --depth;
  errno = saved;
  return result;
}

int snd_pcm_hw_params(snd_pcm_t *pcm, snd_pcm_hw_params_t *params) {
  int outer = depth++ == 0;
  int result = next_params(pcm, params), saved = errno;
  if (!outer) {
    --depth;
    errno = saved;
    return result;
  }
  Pcm *slot = find_pcm(pcm);
  atomic_fetch_add(&slot->params_calls, 1);
  atomic_store(&slot->params_error, (uint64_t)(int64_t)result);
  if (result == 0) {
    snd_pcm_uframes_t buffer = 0, period = 0;
    unsigned int channels = 0, rate = 0;
    int direction = 0;
    int error = next_get_params(pcm, &buffer, &period);
    if (!error)
      error = next_channels(params, &channels);
    if (!error)
      error = next_rate(params, &rate, &direction);
    atomic_store(&slot->params_error, (uint64_t)(int64_t)error);
    atomic_store(&slot->buffer, buffer);
    atomic_store(&slot->period, period);
    atomic_store(&slot->channels, channels);
    atomic_store(&slot->rate, rate);
  }
  --depth;
  errno = saved;
  return result;
}

static void record_call(Calls *calls, uint64_t begin, uint64_t end,
                        int64_t result, uint64_t frames) {
  uint64_t previous =
      atomic_exchange_explicit(&calls->last_ns, begin, memory_order_relaxed);
  atomic_fetch_add_explicit(&calls->count, 1, memory_order_relaxed);
  if (previous && begin >= previous)
    maximum(&calls->max_gap_ns, begin - previous);
  if (end >= begin)
    maximum(&calls->max_duration_ns, end - begin);
  maximum(&calls->max_frames, frames);
  if (result < 0) {
    atomic_fetch_add_explicit(&calls->negative, 1, memory_order_relaxed);
    uint64_t empty = 0;
    atomic_compare_exchange_strong_explicit(
        &calls->first_negative, &empty, (uint64_t)result, memory_order_relaxed,
        memory_order_relaxed);
    if (result == -EPIPE) {
      atomic_fetch_add_explicit(&calls->epipes, 1, memory_order_relaxed);
      empty = 0;
      atomic_compare_exchange_strong_explicit(&calls->first_epipe_ns, &empty,
                                              end, memory_order_relaxed,
                                              memory_order_relaxed);
    }
  }
}

snd_pcm_sframes_t snd_pcm_avail(snd_pcm_t *pcm) {
  int original = errno, outer = depth++ == 0;
  Pcm *slot = outer ? find_pcm(pcm) : NULL;
  uint64_t begin = outer ? now_ns() : 0;
  errno = original;
  snd_pcm_sframes_t result = next_avail(pcm);
  int saved = errno;
  if (outer) {
    uint64_t end = now_ns();
    record_call(&slot->avail, begin, end, result, 0);
    avail_end_ns = end;
  }
  --depth;
  errno = saved;
  return result;
}

snd_pcm_sframes_t snd_pcm_writei(snd_pcm_t *pcm, const void *buffer,
                                 snd_pcm_uframes_t size) {
  int original = errno, outer = depth++ == 0;
  Pcm *slot = outer ? find_pcm(pcm) : NULL;
  uint64_t begin = outer ? now_ns() : 0;
  if (outer && avail_end_ns && begin >= avail_end_ns)
    maximum(&slot->max_processing_ns, begin - avail_end_ns);
  errno = original;
  snd_pcm_sframes_t result = next_write(pcm, buffer, size);
  int saved = errno;
  if (outer) {
    uint64_t end = now_ns();
    record_call(&slot->write, begin, end, result, size);
  }
  --depth;
  errno = saved;
  return result;
}

int snd_pcm_prepare(snd_pcm_t *pcm) {
  int original = errno, outer = depth++ == 0;
  Pcm *slot = outer ? find_pcm(pcm) : NULL;
  errno = original;
  int result = next_prepare(pcm), saved = errno;
  if (outer)
    atomic_fetch_add_explicit(&slot->prepares, 1, memory_order_relaxed);
  --depth;
  errno = saved;
  return result;
}

int snd_pcm_recover(snd_pcm_t *pcm, int error, int silent) {
  int original = errno, outer = depth++ == 0;
  Pcm *slot = outer ? find_pcm(pcm) : NULL;
  errno = original;
  int result = next_recover(pcm, error, silent), saved = errno;
  if (outer)
    atomic_fetch_add_explicit(&slot->recovers, 1, memory_order_relaxed);
  --depth;
  errno = saved;
  return result;
}

int snd_pcm_close(snd_pcm_t *pcm) {
  int original = errno, outer = depth++ == 0;
  Pcm *slot = outer ? find_pcm(pcm) : NULL;
  errno = original;
  int result = next_close(pcm), saved = errno;
  if (outer && result == 0)
    atomic_store(&slot->closed, 1);
  --depth;
  errno = saved;
  return result;
}
