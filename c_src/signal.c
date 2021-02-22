// Copyright 2021 Shin Yoshida
//
// This file is part of Mouse.
//
// Mouse is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License.
//
// Mouse is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Mouse.  If not, see <https://www.gnu.org/licenses/>.

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
