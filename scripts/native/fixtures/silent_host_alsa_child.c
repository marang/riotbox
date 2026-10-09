#define _POSIX_C_SOURCE 200809L
#include <alsa/asoundlib.h>
#include <errno.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>
extern void fixture_mode(const char *);
static const char *mode;
static void *exercise(void *argument) {
  snd_pcm_t *pcm = argument;
  for (int i = 0; i < 6000; ++i) {
    snd_pcm_sframes_t result = snd_pcm_avail(pcm);
    if (errno != 74 || (result != 512 && result != -EPIPE))
      exit(91);
    result = snd_pcm_writei(pcm, NULL, 512);
    if (errno != 75 || (result != 512 && result != -EPIPE))
      exit(92);
  }
  return NULL;
}
int main(int argc, char **argv) {
  /* Give the real supervisor time to bind this generated direct-child PID. */
  struct timespec startup = {.tv_sec = 0, .tv_nsec = 20000000};
  nanosleep(&startup, NULL);
  mode = argc > 1 ? argv[1] : "clean";
  int repetitions = strcmp(mode, "capacity") == 0 ? 65
                    : strcmp(mode, "reuse") == 0  ? 2
                                                  : 1;
  if (strcmp(mode, "capture") == 0) {
    snd_pcm_t *pcm;
    return snd_pcm_open(&pcm, "generated", SND_PCM_STREAM_CAPTURE, 0);
  }
  struct timespec begin, end;
  clock_gettime(CLOCK_MONOTONIC, &begin);
  for (int index = 0; index < repetitions; ++index) {
    snd_pcm_t *pcm;
    if (snd_pcm_open(&pcm, "generated", SND_PCM_STREAM_PLAYBACK, 0) ||
        errno != 71)
      return 93;
    fixture_mode(mode);
    if (snd_pcm_hw_params(pcm, NULL) || errno != 73)
      return 94;
    fixture_mode(mode); /* reset the nested fake availability call */
    if (snd_pcm_prepare(pcm) || errno != 76)
      return 95;
    if (strcmp(mode, "concurrent") == 0) {
      pthread_t thread;
      if (pthread_create(&thread, NULL, exercise, pcm))
        return 96;
      exercise(pcm);
      pthread_join(thread, NULL);
    } else
      exercise(pcm);
    if (snd_pcm_recover(pcm, -EPIPE, 1) || errno != 77)
      return 97;
    if (strcmp(mode, "hang") == 0) {
      puts("ready");
      fflush(stdout);
      pause();
    }
    if (snd_pcm_close(pcm) || errno != 72)
      return 98;
  }
  if (strcmp(mode, "fork_open") == 0) {
    pid_t child = fork();
    if (child < 0)
      return 99;
    if (child == 0) {
      snd_pcm_t *foreign;
      _exit(snd_pcm_open(&foreign, "generated", SND_PCM_STREAM_PLAYBACK, 0));
    }
    int status;
    if (waitpid(child, &status, 0) != child || !WIFEXITED(status))
      return 99;
    return WEXITSTATUS(status);
  }
  clock_gettime(CLOCK_MONOTONIC, &end);
  long long elapsed =
      (end.tv_sec - begin.tv_sec) * 1000000000LL + end.tv_nsec - begin.tv_nsec;
  printf("%lld\n", elapsed);
  return 0;
}
