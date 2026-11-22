"""Persisted training time excludes corpus preparation, validation and recovery."""

import time


class TrainingClock:
    def __init__(self, elapsed=0.0, now=time.monotonic):
        self.elapsed = float(elapsed)
        self.now = now
        self.started = None

    def seconds(self):
        return self.elapsed + (0.0 if self.started is None else self.now() - self.started)

    def resume(self):
        if self.started is None:
            self.started = self.now()

    def pause(self):
        self.elapsed = self.seconds()
        self.started = None
