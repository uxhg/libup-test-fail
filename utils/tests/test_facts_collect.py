import logging
import os
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from clone_co import clone_co, init_logging
from common import ClientAtVer, add_suffix
from run_depgraph import single_client

logger = logging.getLogger(__name__)


class TestFactsCollect(unittest.TestCase):
    def test_clone_co(self):
        c = ClientAtVer(name="commons-csv", url="https://github.com/apache/commons-csv",
                        sha1="660f7c9f853092ec8abf5d6c81d260e3c80c2194")
        with TemporaryDirectory() as tempdir:
            repo, local_path = clone_co(c, Path(tempdir))
            logger.info(repo)
            self.assertEqual(local_path, Path(tempdir) / add_suffix(c.name))
            self.assertEqual(next(repo.remotes[0].urls), c.url,
                             "repo returned by clone_co() did not show the correct remote url")
            self.assertEqual(str(repo.head.commit), c.sha1,
                             "clone_co() possibly did not check out to the correct commit")

    def test_run_single_client(self):
        c = ClientAtVer(name="commons-csv", url="https://github.com/apache/commons-csv",
                        sha1="660f7c9f853092ec8abf5d6c81d260e3c80c2194")
        with TemporaryDirectory() as tempdir:
            print(tempdir)
            single_client(c, Path(tempdir))


if __name__ == '__main__':
    init_logging("info")
    unittest.main()
