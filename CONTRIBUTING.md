# Reporting Issues
If you encounter any issues while using `the-heck`, or if you have suggestions for new features, please add an issue.

For bugs, please include the following information in your issue description:
1. The shell that you're using (e.g. `bash`, `zsh`, Windows PowerShell),
2. Your system (e.g. `Ubuntu 20.04`, `Windows`, `MacOS`),
3. How to reproduce the bug,
4. The full error message (if applicable),
5. Confirmation that you are on the most recent stable version.


# Making a pull request
If you'd like to address any of the current issues, add new features, etc., please open a pull request!

The general workflow is as follows:
1. Make a fork of this repository
2. Clone it to your local machine
3. Set origin and upstream (if not set by default)
4. Create a feature branch with the following branch name style:
```
{your-github-username}/{type-of-contribution}/{short-description}

For example:
elizabeth-oda/feat/add-powershell-support
elizabeth-oda/fix/error-handle-one-arg-commands
```
5. Develop your feature or fix
6. Write relevant tests *and* describe in your PR how to run the tests
7. Open a PR from your fork to this repository. Assign @elizabeth-oda as a reviewer.
