#include <errno.h>
#include <signal.h>
#include <string.h>

#ifndef NULL
#define NULL 0
#endif

/**
 * Wait for signal SIGHUP, SIGINT, and SIGTERM.
 *
 * On success, returns 0; otherwise, set errno and returns 1.
 */
int sigwait_() {
  if (errno)
    return 1;

  sigset_t ss;
  if (sigemptyset(&ss))
    return 1;

  if (sigaddset(&ss, SIGHUP) || sigaddset(&ss, SIGINT) ||
      sigaddset(&ss, SIGTERM))
    return 1;

  if (sigprocmask(SIG_BLOCK, &ss, NULL))
    return 1;

  int sig;
  if (sigwait(&ss, &sig))
    return 1;

  return 0;
}
