import datetime
import logging
import os
import sys
from pathlib import Path
from typing import NamedTuple

THIS_DIR = Path(os.path.dirname(os.path.realpath(__file__)))
ALL_CLIENTS_JSON: Path = (THIS_DIR / "../depgraph/data/external/all-clients.json").resolve()
ALL_PAIRS_JSON: Path = (THIS_DIR / "../depgraph/data/external/incompat-pairs-all.json").resolve()
ONE_PAIR_JSON: Path = (THIS_DIR / "../depgraph/data/for-tests/incompat-pairs-one.json")
LOC_REPO: Path = (THIS_DIR / "../../cases").resolve()


class ClientAtVer(NamedTuple):
    name: str
    url: str
    sha1: str


class ColorFormatter(logging.Formatter):
    COLORS = {
        logging.DEBUG: "\033[96m",
        logging.INFO: "\033[92m",
        logging.WARNING: "\033[93m",
        logging.ERROR: "\033[91m",
        logging.CRITICAL: "\033[01;91m\033[47m",  # bold red on white background
        'RESET': "\033[0m"
    }

    def format(self, record):
        color = self.COLORS[record.levelno]
        color_reset = self.COLORS["RESET"]
        self.datefmt = "%m-%d %H:%M:%S"
        self._style._fmt = color + '[%(asctime)s] [%(levelname)8s] ' + color_reset + '%(message)s'
        return super().format(record)


def init_logging():
    root_logger = logging.getLogger()
    log_level = os.environ.get("LOG_LEVEL")
    if not log_level:
        log_level = "warning"
    numeric_level = getattr(logging, log_level.upper(), None)
    if not isinstance(numeric_level, int):
        raise ValueError('Invalid log level: %s' % log_level)
    root_logger.setLevel(numeric_level)
    handler = logging.StreamHandler(sys.stderr)
    handler.setFormatter(ColorFormatter())
    root_logger.addHandler(handler)


def get_cur_time_str() -> str:
    return str(datetime.datetime.now().isoformat()).replace(':', '-')


def add_suffix(x: str) -> str: return x + "-clean"


def rm_suffix(x: str) -> str: return x.rsplit("-clean", 1)[0]
