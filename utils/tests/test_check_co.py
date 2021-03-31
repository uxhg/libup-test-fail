import logging
import unittest

from git import Repo

from clone_co import clone_co
from common import ClientAtVer
from pathlib import Path
from tempfile import TemporaryDirectory

logger = logging.getLogger(__name__)


class TestCloneAndCheckout(unittest.TestCase):
    def test_clone_co(self):
        c = ClientAtVer(name="commons-csv", url="https://github.com/apache/commons-csv",
                        sha1="660f7c9f853092ec8abf5d6c81d260e3c80c2194")
        with TemporaryDirectory() as tempdir:
            repo: Repo = clone_co(c, Path(tempdir))
            logger.info(repo)
            self.assertEqual(next(repo.remotes[0].urls), c.url,
                             "repo returned by clone_co() did not show the correct remote url")
            self.assertEqual(str(repo.head.commit), c.sha1,
                             "clone_co() possibly did not check out to the correct commit")


if __name__ == '__main__':
    unittest.main()
