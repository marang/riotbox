/* Generated ABI control: exports ALSA signatures but never opens ALSA/audio. */
#define _POSIX_C_SOURCE 200809L
#include <alsa/asoundlib.h>
#include <errno.h>
#include <stdatomic.h>
#include <stdint.h>
#include <string.h>
static const char *mode = "clean";
static _Atomic unsigned int avails, writes;
void fixture_mode(const char *value) {
  mode = value;
  avails = 0;
  writes = 0;
}
int snd_pcm_open(snd_pcm_t **pcm, const char *name, snd_pcm_stream_t stream,
                 int flags) {
  (void)name;
  (void)stream;
  (void)flags;
  *pcm = (snd_pcm_t *)(uintptr_t)0x1000;
  errno = 71;
  return 0;
}
int snd_pcm_close(snd_pcm_t *pcm) {
  (void)pcm;
  errno = 72;
  return 0;
}
int snd_pcm_hw_params(snd_pcm_t *pcm, snd_pcm_hw_params_t *params) {
  (void)pcm;
  (void)params;
  /* Nested unknown inner handle must be forwarded but not attributed to CPAL.
   */
  snd_pcm_avail((snd_pcm_t *)(uintptr_t)0xdead);
  errno = 73;
  return 0;
}
int snd_pcm_get_params(snd_pcm_t *pcm, snd_pcm_uframes_t *buffer,
                       snd_pcm_uframes_t *period) {
  (void)pcm;
  *buffer = 4096;
  *period = 512;
  return strcmp(mode, "geometry_error") == 0 ? -EINVAL : 0;
}
int snd_pcm_hw_params_get_channels(const snd_pcm_hw_params_t *params,
                                   unsigned int *value) {
  (void)params;
  *value = 2;
  return 0;
}
int snd_pcm_hw_params_get_rate(const snd_pcm_hw_params_t *params,
                               unsigned int *value, int *dir) {
  (void)params;
  *value = 44100;
  *dir = 0;
  return 0;
}
snd_pcm_sframes_t snd_pcm_avail(snd_pcm_t *pcm) {
  (void)pcm;
  unsigned int call = atomic_fetch_add(&avails, 1);
  errno = 74;
  return strcmp(mode, "avail_error") == 0 && call == 0 ? -EPIPE : 512;
}
snd_pcm_sframes_t snd_pcm_writei(snd_pcm_t *pcm, const void *buffer,
                                 snd_pcm_uframes_t size) {
  (void)pcm;
  (void)buffer;
  unsigned int call = atomic_fetch_add(&writes, 1);
  errno = 75;
  return strcmp(mode, "write_error") == 0 && call == 0
             ? -EPIPE
             : (snd_pcm_sframes_t)size;
}
int snd_pcm_prepare(snd_pcm_t *pcm) {
  (void)pcm;
  errno = 76;
  return 0;
}
int snd_pcm_recover(snd_pcm_t *pcm, int error, int silent) {
  (void)error;
  (void)silent;
  snd_pcm_prepare(pcm); /* nested prepare must not inflate top-level counts */
  errno = 77;
  return 0;
}
