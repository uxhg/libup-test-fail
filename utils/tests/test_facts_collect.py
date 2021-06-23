import logging
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from clone_co import clone_co, init_logging
from common import ClientAtVer, add_suffix, ONE_PAIR_JSON
from run_depgraph import single_client, get_module_list, run_all

logger = logging.getLogger(__name__)

TestClient = ClientAtVer(name="commons-csv", url="https://github.com/apache/commons-csv",
                         sha1="660f7c9f853092ec8abf5d6c81d260e3c80c2194")


class TestFactsCollect(unittest.TestCase):
    def test_clone_co(self):
        c = TestClient
        with TemporaryDirectory() as tempdir:
            repo, local_path = clone_co(c, Path(tempdir))
            print(repo)
            self.assertEqual(local_path, Path(tempdir) / add_suffix(c.name))
            self.assertEqual(next(repo.remotes[0].urls), c.url,
                             "repo returned by clone_co() did not show the correct remote url")
            self.assertEqual(str(repo.head.commit), c.sha1,
                             "clone_co() possibly did not check out to the correct commit")

    def test_run_single_client(self):
        with TemporaryDirectory() as tempdir:
            print(tempdir)
            failed_set: set = single_client(TestClient, Path(tempdir))
            self.assertEqual(len(failed_set), 0)

    def test_mvn_list_mod_single(self):
        with TemporaryDirectory() as tempdir:
            print(tempdir)
            _, local_path = clone_co(TestClient, Path(tempdir))
            mods = get_module_list(local_path)
            self.assertEqual(len(mods), 1, "Should be one module")
            self.assertEqual(mods[0], f"{Path(tempdir) / 'commons-csv-clean'}")
            print(mods[0])

    def test_run_all_using_one(self):
        run_all(ONE_PAIR_JSON)


if __name__ == '__main__':
    init_logging("info")
    unittest.main()
