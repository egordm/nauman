<div id="top"></div>

# Job Syntax
The job file is a YAML file that describes the job to be run. It is heavily inspired by [Github Actions Workflow](https://docs.github.com/en/actions/learn-github-actions/workflow-syntax-for-github-actions) files, but contains some differences. Documentation is therefore provided in a similar fashion with `<job>` as root key (referring to the job file itself).

## Table Of Contents
* [Jobs](#jobs)
    + [`<job>.id`](#-jobid-)
    + [`<job>.name`](#-jobname-)
    + [`<job>.env`](#-jobenv-)
    + [`<job>.cwd`](#-jobcwd-)
    + [`<job>.policy`](#-jobpolicy-)
    + [`<job>.hooks`](#-jobhooks-)
* [Tasks](#tasks)
    + [`<job>.tasks.<task>.id`](#-jobtasks-task-id-)
    + [`<job>.tasks.<task>.name`](#-jobtasks-task-name-)
    + [`<job>.tasks.<task>.env`](#-jobtasks-task-env-)
    + [`<job>.tasks.<task>.cwd`](#-jobtasks-task-cwd-)
    + [`<job>.tasks.<task>.run`](#-jobtasks-task-run-)
    + [`<job>.tasks.<task>.shell`](#-jobtasks-task-shell-)
    + [`<job>.tasks.<task>.shell_path`](#-jobtasks-task-shell-path-)
    + [`<job>.tasks.<task>.policy`](#-jobtasks-task-policy-)
    + [`<job>.tasks.<task>.hooks`](#-jobtasks-task-hooks-)
* [Logging](#logging)
    + [`<job>.logging.<log>.type`](#-joblogging-log-type-)
    + [`<job>.logging.<log>.name`](#-joblogging-log-name-)
    + [`<job>.logging.<log>.stdout`](#-joblogging-log-stdout-)
    + [`<job>.logging.<log>.stderr`](#-joblogging-log-stderr-)
    + [`<job>.logging.<log>.hooks`](#-joblogging-log-hooks-)
    + [`<job>.logging.<log>.internal`](#-joblogging-log-internal-)
    + [File logging](#file-logging)
    + [`<job>.logging.<log>.file`](#-joblogging-log-file-)
    + [`<job>.logging.<log>.split`](#-joblogging-log-split-)
    + [Console logging](#console-logging)
* [Global Options](#global-options)
    + [`<job>.options.shell`](#-joboptionsshell-)
    + [`<job>.options.shell_path`](#-joboptionsshell-path-)
    + [`<job>.options.dry_run`](#-joboptionsdry-run-)
    + [`<job>.options.ansi`](#-joboptionsansi-)
    + [`<job>.options.log_level`](#-joboptionslog-level-)
    + [`<job>.options.log_dir`](#-joboptionslog-dir-)
    + [`<job>.options.system_env`](#-joboptionssystem-env-)

## Jobs

### `<job>.id`
The job id is a string that uniquely identifies the job. It is used to identify the job in the logs. By default, it is set to the name of the job file.

### `<job>.name`
The job name is a string that is used to display the job in the logs or other output. By default, it is set to the name of the job file.

### `<job>.env`
The job env is a list of environment variables that will be set before the job is run. They are also used for each job.

```yaml
env:
  FOO: bar
  BAZ: qux
```

### `<job>.cwd`
The job cwd is a string that is used to set the current working directory before the job is run. All the other relative paths used in the job are relative to this directory.

### `<job>.policy`
The job policy is the global execution policy enforced for all the tasks unless overridden. It is a string that can be one of the following:

* `always` - Always execute the task regardless of prior task status.
* `prior_success` - Execute the task only if prior task has succeeded.
* `no_prior_failed` - Execute the task only if no other task has failed.

### `<job>.hooks`
The global hooks are a list of hooks that apply to all the tasks. Global before hooks have always higher precedence while after hooks have the lowest precedence when task specific hooks are involved. Each hook is list of tasks and can be one of the following:

```yaml
hooks:
  before_job:
    ...
  after_job:
    ...
  on_failure:
    ...
  on_success:
    ...
  before_task:
    ...
  after_task:
    ...
```

<p align="right">(<a href="#top">back to top</a>)</p>

## Tasks

### `<job>.tasks.<task>.id`
The task id is a string that uniquely identifies the task. It is used to identify the task in the logs. By default, it is set as transformed task name or command (run) name.

### `<job>.tasks.<task>.name`
The task name is a string that is used to display the task in the logs or other output.

### `<job>.tasks.<task>.env`
The task env is a list of environment variables that will be set before the task is run. They are also used for the task and merged with all the other env variables.

```yaml
tasks:
  - name: run
    env:
      FOO: bar
      BAZ: qux
```

### `<job>.tasks.<task>.cwd`
The task cwd is a string that is used to set the current working directory before the task is run. All the other relative paths used in the task are relative to this directory.

### `<job>.tasks.<task>.run`
The task `run` argument is a string that refers to a command to run. It should be a program valid within the given shell.

```yaml
tasks:
  - name: single line
    run: echo "Hello World"
  - name: multiline
    run: |
      echo "Hello World"
      echo "Hello World"
```

### `<job>.tasks.<task>.shell`
The shell is a string that is used to specify the shell to use for the tasks.

The default is `sh`. But, you can choose any of the following:

* `bash` - Bash shell.
* `python` - Python shell.
* `ruby` - Ruby shell.
* `php` - Php shell.
* `node` - Node shell.
* `cmd` - Windows command shell.
* `powershell` - PowerShell shell.

This option refers only to shell type. If you want to use a specific shell, you can use the `shell_path` option.


### `<job>.tasks.<task>.shell_path`
The shell path is a string that is used to specify the path to the shell to use for the tasks. If not specified, the shell is determined by the ones available in the system.

### `<job>.tasks.<task>.policy`
The task policy is the execution policy enforced for the task. It is a string that can be one of the following:

* `always` - Always execute the task regardless of prior task status.
* `prior_success` - Execute the task only if prior task has succeeded.
* `no_prior_failed` - Execute the task only if no other task has failed.

### `<job>.tasks.<task>.hooks`
The task-specific hooks are a list of hooks that apply to the specified task. Each hook is list of tasks and can be one of the following:

```yaml
tasks:
  - name: My Task
    hooks:
      on_failure:
        ...
      on_success:
        ...
      before_task:
        ...
      after_task:
        ...
```

<p align="right">(<a href="#top">back to top</a>)</p>

## Logging

### `<job>.logging.<log>.type`
The log type is a string that is used to specify the type of the log. It is one of the following:

* `console` - Log to the console.
* `file` - Log to a file.

### `<job>.logging.<log>.name`
The logging name is a string that is used to display the logging option in the logs or other output.

### `<job>.logging.<log>.stdout`
If set to `true`, the standard output of the task will be captured and logged.

Default: `true`

### `<job>.logging.<log>.stderr`
If set to `true`, the standard error of the task will be captured and logged.

Default: `true`

### `<job>.logging.<log>.hooks`
If set to `true`, the standard output and error of the hook tasks will also be captured and logged.

Default: `true`

### `<job>.logging.<log>.internal`
If set to `true`, the log output of `nauman` will also be captured and logged.

Default: `true`

### File logging
### `<job>.logging.<log>.file`
Refers to the file path of the file to store the log into.

If `split` is set to `true`, this file should refer to a directory. The log will be stored in a file named after the task id within this directory.

If a relative path is given, then it is relative to the log directory.

### `<job>.logging.<log>.split`
If set to `true`, the log will be stored in a file named after the task id within the specified directory.

### Console logging
none

<p align="right">(<a href="#top">back to top</a>)</p>

## Global Options
### `<job>.options.shell`
The shell is a string that is used to specify the shell to use for the tasks.

The default is `sh`. But, you can choose any of the following:

* `bash` - Bash shell.
* `python` - Python shell.
* `ruby` - Ruby shell.
* `php` - Php shell.
* `node` - Node shell.
* `cmd` - Windows command shell.
* `powershell` - PowerShell shell.

This option refers only to shell type. If you want to use a specific shell, you can use the `shell_path` option.

### `<job>.options.shell_path`
The shell path is a string that is used to specify the path to the shell to use for the tasks. If not specified, the shell is determined by the ones available in the system.

### `<job>.options.dry_run`
If set to `true`, the job will always execute in dry run mode.

### `<job>.options.ansi`
If set to `false`, the job will not output ANSI escape codes.

### `<job>.options.log_level`
The log level is a string that is used to specify the log level. It is one of the following:

* `debug` - Debug level.
* `info` - Info level.
* `warn` - Warn level.
* `error` - Error level.

### `<job>.options.log_dir`
The log directory is a string that is used to specify the directory to store the logs. If not specified, the logs will be stored in the current working directory.

### `<job>.options.system_env`
If set to `true`, the job will use the system environment variables. If set to `false`, the job will only use the environment variables explicitly defined in the job, task or in the cli.


<p align="right">(<a href="#top">back to top</a>)</p>