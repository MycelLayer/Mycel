import json
import subprocess
import unittest
from pathlib import Path
from unittest import mock

import scripts.github_repo_context as repo_context


class GitHubRepoContextTest(unittest.TestCase):
    def test_parse_repo_slug_from_https_remote(self) -> None:
        self.assertEqual(
            "example-org/example-repo",
            repo_context.parse_repo_slug_from_remote("https://github.com/example-org/example-repo.git"),
        )

    def test_parse_repo_slug_from_ssh_remote(self) -> None:
        self.assertEqual(
            "example-org/example-repo",
            repo_context.parse_repo_slug_from_remote("git@github.com:example-org/example-repo.git"),
        )

    def test_resolve_repo_slug_prefers_explicit_value(self) -> None:
        self.assertEqual(
            "explicit-org/explicit-repo",
            repo_context.resolve_repo_slug("explicit-org/explicit-repo"),
        )

    def test_resolve_repo_slug_uses_git_remote_before_gh(self) -> None:
        with mock.patch.object(
            repo_context,
            "repo_slug_from_git_remote",
            return_value="example-org/from-git",
        ) as git_mock:
            with mock.patch.object(repo_context, "repo_slug_from_gh", return_value="example-org/from-gh") as gh_mock:
                slug = repo_context.resolve_repo_slug(None)

        self.assertEqual("example-org/from-git", slug)
        git_mock.assert_called_once()
        gh_mock.assert_not_called()

    def test_repo_slug_from_gh_reads_name_with_owner(self) -> None:
        with mock.patch.object(subprocess, "run") as run_mock:
            run_mock.return_value = subprocess.CompletedProcess(
                args=[],
                returncode=0,
                stdout=json.dumps({"nameWithOwner": "example-org/example-repo"}),
                stderr="",
            )

            slug = repo_context.repo_slug_from_gh(cwd=Path("."))

        self.assertEqual("example-org/example-repo", slug)

    def test_resolve_repo_slug_raises_when_context_is_missing(self) -> None:
        with mock.patch.object(repo_context, "repo_slug_from_git_remote", return_value=None):
            with mock.patch.object(repo_context, "repo_slug_from_gh", return_value=None):
                with self.assertRaises(repo_context.RepoContextError):
                    repo_context.resolve_repo_slug(None)


if __name__ == "__main__":
    unittest.main()
