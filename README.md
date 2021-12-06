<!-- markdownlint-disable -->
<div align="center">
    <h1>nauman</h1>
    <p>
       <img alt="Crates.io" src="https://img.shields.io/crates/v/nauman">
       <a href="LICENSE"><img src="https://img.shields.io/github/license/EgorDm/nauman" alt="License"></a>
 <a href="https://github.com/EgorDm/nauman/actions/workflows/ci.yml"><img src="https://github.com/EgorDm/nauman/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
    </p>
    <p>
        <b>A CI inspired approach for local job automation.</b>
    </p>
</div>
<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#faq">FAQ</a> •
  <a href="#examples">Examples</a> •
  <a href="#job-syntax">Job Syntax</a>
</p>
<!-- markdownlint-enable -->


## About
`nauman` is an easy-to-use job automation tool. It arose from a necessity to automate complex task flows while still preserving the ability to monitor and debug them.

It is heavily inspired by simplicity of [Github Actions](https://docs.github.com/en/actions), flexibility of [Fastlane](https://github.com/fastlane/fastlane) and extensibility of [Apache Airflow](https://airflow.apache.org/). This tool aims to bring the best of both to local job automation.

## Quick Start
See [Installation](#installation) for how to install just on your computer. Try running `nauman --version` to make sure that it’s installed correctly.

Once `nauman` is installed and working, create a job file named `hello-world.yml` in the root of your project with the following contents:

```yaml
name: Hello World!

tasks:
  - name: Hello World!
    run: echo "Hello World!"
  - name: Greeting
    run: echo "Greetings ${USER}!"
```

When you invoke `nauman hello-world.yml` it runs the job tasks in the order they are listed in the file. The output should be as follows:

<pre>
<span style="color: #50FA7B">--------------------------
--- Task: Hello World! ---
--------------------------</span>
<span style="color: #8BE9FD">$ echo "Hello World!"</span>
Hello World!
<span style="color: #50FA7B">----------------------
--- Task: Greeting ---
----------------------</span>
<span style="color: #8BE9FD">$ echo "Greetings ${USER}!"</span>
Greetings egordm!

</pre>

`nauman` prints the output of each task to the console. The defined tasks run within your default shell and capture all of their output.

## Examples
For more examples, see the [examples](examples) directory.

### Using Hooks
[Hooks](#hooks) are first class citizens in `nauman`. They represent various events and callbacks that can occur during the execution of a job.

Let's take a look at a simple use case of hooks to add health checks to a job and its tasks. Create a file named `health-checks.yml` in the root of your project with the following contents:

```yaml
name: Example Job Using Health Checks
policy: always

tasks:
  - name: Run a successful program
    run: sleep 2 && echo "Success!"
    hooks:
      on_success:
        - run: curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/fb4c4863-a7f1-44f1-8298-3baabec653d4
  - name: Run a failing program
    run: sleep 2 && exit 1
    hooks:
      on_success:
        - run: curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/0178d446-9b50-4158-b50d-7df098945c81
      on_failure:
        - name: Send failing status code to Health Check
          run: curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/0178d446-9b50-4158-b50d-7df098945c81/$NAUMAN_PREV_CODE

hooks:
  after_job:
    - name: On completion of the job, ping a health check
      run: curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/fb4c4863-a7f1-44f1-8298-3baabec653d4
```

When you invoke `nauman health-checks.yml` it runs all the tasks withing the job file despite the fact that the second task fails (see [Execution Policy: always](#execution-policies)). See the output below:

<pre>
<span style="color: #50FA7B">--------------------------------------
--- Task: Run a successful program ---
--------------------------------------</span>
<span style="color: #8BE9FD">$ sleep 2 &amp;&amp; echo "Success!"</span>
Success!
<span style="color: #F1FA8C">-------------------------------------------------------------------------------------------------------------
--- Hook: curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/fb4c4863-a7f1-44f1-8298-3baabec653d4 ---
-------------------------------------------------------------------------------------------------------------</span>
<span style="color: #8BE9FD">$ curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/fb4c4863-a7f1-44f1-8298-3baabec653d4</span>
<span style="color: #50FA7B">-----------------------------------
--- Task: Run a failing program ---
-----------------------------------</span>
<span style="color: #8BE9FD">$ sleep 2 &amp;&amp; exit 1</span>
<span style="color: #FF5555">Task "Run a failing program" completed in 2s with a non-zero exit status: 1. This indicates a failure</span>
<span style="color: #F1FA8C">------------------------------------------------------
--- Hook: Send failing status code to Health Check ---
------------------------------------------------------</span>
<span style="color: #8BE9FD">$ curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/0178d446-9b50-4158-b50d-7df098945c81/$NAUMAN_PREV_CODE</span>
<span style="color: #F1FA8C">-----------------------------------------------------------
--- Hook: On completion of the job, ping a health check ---
-----------------------------------------------------------</span>
<span style="color: #8BE9FD">$ curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/fb4c4863-a7f1-44f1-8298-3baabec653d4</span>
</pre>

On success of the first task, a success hook is executed which sends a health-check. On failure of the second task, a fail hook is executed sending a failure health-check. Finally, an after job hook is executed sending a job completion health-check.

### Logging
[Logging](#logging) is a powerful feature of `nauman` that allows you to log the output of your tasks and hooks to different output streams.

Create a file named `logging.yml` in the root of your project with the following contents:

```yaml
name: Example Job Using Logs
options:
  log_dir: ./logs

tasks:
  - name: Print Hello World to stdout
    run: echo "Hello World!"
  - name: Print Hello World to stderr
    run: echo "Hello World!" >&2

logging:
  - type: file
    name: Print stdout to a file
    stdout: true
    stderr: false
    output: ./stdout.log
  - type: file
    name: Print stderr to a file
    stdout: false
    stderr: true
    output: ./stderr.log
  - type: file
    name: Print both stdout and stderr to separate files per task
    split: true
    output: ./separate_logs
  - type: console
```

Run `nauman logging.yml` and see the output below:

<pre>
<span style="color: #50FA7B">-----------------------------------------
--- Task: Print Hello World to stdout ---
-----------------------------------------</span>
<span style="color: #8BE9FD">$ echo "Hello World!"</span>
Hello World!
<span style="color: #50FA7B">-----------------------------------------
--- Task: Print Hello World to stderr ---
-----------------------------------------</span>
<span style="color: #8BE9FD">$ echo "Hello World!" &gt;&amp;2</span>
Hello World!
</pre>

Additionally, the following files are created:
* `logs/logging_2021-12-05T18:11:14/`
  * `separate_logs/`
    * `000_print-hello-world-to-stdout.log`
    * `001_print-hello-world-to-stderr.log`
  * `stderr.log`
  * `stdout.log`

Where the logs if the specified root directory for the logs (See `log_dir` in [Logging](#logging) for more details). All the logs are placed in an `logging_` subdirectory with the current date and time of the job run.
`stdout.log` and `stderr.log` are created for each log stream. 
`separate_logs/` is created for each task and contains the stdout and stderr logs for that task.

### Using Environment Variables
[Environment Variables](#environment-variables) allow you to set environment variables for your job. There are multiple ways to set environment variables:

* As system environment variables: `KEY=VALUE nauman`
* As cli arguments: `nauman -e KEY=VALUE`
* As job configuration: `<job_file>.env.KEY: VALUE`
* As task configuration: `<job_file>.tasks.<task>.env.KEY: VALUE`

By creating `env-vars.yml` in the root of your project with the following content you can test them all:

```yaml
name: Example Environment variables

env:
  PING_CMD: curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/
  CHECK_1: fb4c4863-a7f1-44f1-8298-3baabec653d4

tasks:
  - name: Job env var
    run: echo $PING_CMD$CHECK_1
  - name: Task env var
    run: echo $PING_CMD$CHECK_1
    env:
      CHECK_1: fb4c4863-a7f1-44f1-8298-3baabec653d4
  - name: System env var
    run: echo $PING_CMD$CHECK_2
  - name: Built-in env vars
    run: echo "Previous task \"$NAUMAN_PREV_NAME\" finished with status $NAUMAN_PREV_CODE"
```

When you run `nauman env-vars.yml -e CHECK_2=0178d446-9b50-4158-b50d-7df098945c81` you will see the following output:

<pre>
<span style="color: #50FA7B">-------------------------
--- Task: Job env var ---
-------------------------</span>
<span style="color: #8BE9FD">$ echo $PING_CMD$CHECK_1</span>
curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/fb4c4863-a7f1-44f1-8298-3baabec653d4
<span style="color: #50FA7B">--------------------------
--- Task: Task env var ---
--------------------------</span>
<span style="color: #8BE9FD">$ echo $PING_CMD$CHECK_1</span>
curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/fb4c4863-a7f1-44f1-8298-3baabec653d4
<span style="color: #50FA7B">----------------------------
--- Task: System env var ---
----------------------------</span>
<span style="color: #8BE9FD">$ echo $PING_CMD$CHECK_2</span>
curl -fsS -m 10 --retry 5 -o /dev/null https://hc-ping.com/0178d446-9b50-4158-b50d-7df098945c81
<span style="color: #50FA7B">-------------------------------
--- Task: Built-in env vars ---
-------------------------------</span>
<span style="color: #8BE9FD">$ echo "Previous task \"$NAUMAN_PREV_NAME\" finished with status $NAUMAN_PREV_CODE"</span>
Previous task "System env var" finished with status 0
</pre>

In the last task we can see that the `NAUMAN_PREV_NAME` and `NAUMAN_PREV_CODE` environment variables are used. These variables are set by the `nauman` based on the previous task. See [Environment Variables](#environment-variables) for more context specific environment variables.

## Features

### Hook everything
You can create hooks for all the possible outcomes and events of your job or your task. Create job or task-local hooks like this:

```yaml
tasks:
  ...
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

### Flexible Logging
You can log to single or multiple files, to console and even choose which log streams to used (stdout, stderr, or both).

```yaml
logging:
  - name: Log only stdout
    type: file
    stdout: true
    stderr: false
    output: ./stdout.log
  - name: Logs split in files per task
    type: file
    stdout: true
    stderr: true
    split: true
    output: ./per_task_logs
  - name: Logs to console
    type: console
    stdout: true
    stderr: true
  - name: Append output to a shared file
    type: file
    stdout: true
    stderr: true
    output: /var/log/nauman/my_job.log
```

### Context variables
Define more flexible tasks by using context variables.

Currently, following context variables are supported:
* `NAUMAN_JOB_NAME` - Name of the job
* `NAUMAN_JOB_ID` - ID of the job
* `NAUMAN_TASK_NAME` - Name of the current task
* `NAUMAN_TASK_ID` - ID of the current task
* `NAUMAN_PREV_NAME` - Name of the previous task
* `NAUMAN_PREV_ID` - ID of the previous task
* `NAUMAN_PREV_CODE` - Exit code of the previous task

```yaml
tasks:
  ...
  - name: Use context vars as env vars
    run: echo $NAUMAN_TASK_NAME
```

### Configurable task plan
When one task fails it does not stop the whole job. You can configure the task execution plan to decide how to proceed.

You can choose between the following options:
* `always` - Always execute the task regardless of prior task status.
* `prior_success` - Execute the task only if prior task has succeeded.
* `no_prior_failed` - Execute the task only if no other task has failed.

```yaml
# Policy can be defined at job level
policy: no_prior_failed

tasks:
  ...
  - name: Always run this task
    # And overridden at task level
    policy: always
```

### Different shell types
Aside from the default `sh` shell you can use `bash`, `python`, `ruby`, `php` or specify path to your own desired shell.

```yaml
# Specify a default shell
shell: bash
shell_path: /bin/bash

tasks:
  ...
  - name: Python task
    shell: python
    run: print('Hello World!')
  - name: Virtual env python
    shell: python
    shell_path: '/app/venv/bin/python'
    run: print('Hello World!')
  - name: Ruby task
    shell: ruby
    run: print('Hello World!')
  - name: PHP task
    shell: php
    run: echo 'Hello World!';
```

### Dry run
Want to make sure that your job is configured correctly? You can run your job in dry run mode. This will verify that all tasks are syntactically correct, all shells are usable and warn you about any potential issues (such as missing directories).

```shell
nauman --dry-run my_job.yml
```

### Multiline commands
Sometimes commands can take up more space than a single line. You can use multiline strings to define your commands.

```yaml
tasks:
  ...
  - name: Multiline
    shell: python
    run: |
      import os
      print(os.environ['NAUMAN_TASK_NAME'])
```

### Change your working directory
You can change your working directory by using the `cwd` option.

```yaml
cwd: /my/project/dir

tasks:
  ...
  - name: Change working directory to /my/project/dir/task1
    cwd: ./task1
    run: pwd
```

## FAQ

### Why use nauman?
Picture this: you want to periodically run your tool that syncs your favorite movies between services. This can be done with a cron job, but what if you want, to add more dependent tasks (like, also syncing your movie collections)? Easy, create a shell script that runs them both.

Now you want to keep track of their output (for debugging), you want to add health-checks, single process locking, etc. Shell scripts are not the best way to do this and can easily get very messy.

With `nauman` you can create and run a job file that covers it all in a readable and maintainable way.

Additionally `nauman` is written in Rust and can be installed bloat free onto any system as a simple binary. (See [Installation](#installation) for more details).

### When not to use nauman?
You should not use `nauman` for tasks where you need:

* A makefile:
  * `nauman` is not meant to be a replacement for makefiles.
  * It is meant to run a job to automate one single chain of tasks.
  * It does not support task parallelism, recursion or other complex workflows.
* A data automation tool:
  * `nauman` is not meant to be a replacement for data automation tools.
  * It can be used to chain multiple data processing tasks together.
  * But it does not provide anything for data loading, data processing or visualization.
* A CI tool:
  * `nauman` is not meant to be a replacement for CI tools.
  * It does not include any CI-specific features such as caching, build uploads or integrations with build tools.

## Installation
The binary name for nauman is `nauman`.

[Archives of precompiled binaries for nauman are available for Windows, macOS and Linux. Linux and Windows binaries are static executables.](https://github.com/EgorDm/nauman/releases) Users of platforms not explicitly mentioned below are advised to download one of these archives.

If you're a Rust programmer, nauman can be installed with cargo.

* Note that numane is tested with Rust 1.57.0, although nauman may work with older versions.
* Note that the binary may be bigger than expected because it contains debug symbols. This is intentional. To remove debug symbols and therefore reduce the file size, run strip on the binary.

```shell
$ cargo install nauman
```

### Building from source
nauman is written in Rust, so you'll need to grab a [Rust installation](https://www.rust-lang.org/) in order to compile it. nauman compiles with Rust 1.57.0 (stable) or newer. In general, nauman tracks the latest stable release of the Rust compiler.

To build nauman:
```shell
$ git clone https://github.com/EgorDm/nauman
$ cd nauman
$ cargo build --release
$ ./target/release/nauman --version
```

## Usage
The usual way to invoke `nauman` is to use the `nauman <job_file>` command. If you want to specify more options or to override some job settings, refer to the below full usage:

<pre>
<span style="color: #F1FA8C">USAGE:</span>
    nauman [OPTIONS] &lt;JOB&gt;

<span style="color: #F1FA8C">ARGS:</span>
    <span style="color: #50FA7B">&lt;JOB&gt;</span>    Path to job yaml file

<span style="color: #F1FA8C">OPTIONS:</span>
        <span style="color: #50FA7B">--ansi</span> <span style="color: #50FA7B">&lt;ANSI&gt;</span>                Include ansi colors in output (default: true)
        <span style="color: #50FA7B">--dry-run</span> <span style="color: #50FA7B">&lt;DRY_RUN&gt;</span>          Dry run to check job configuration (default: false)
    <span style="color: #50FA7B">-e</span> <span style="color: #50FA7B">&lt;ENV&gt;</span>                         List of env variable overrides
    <span style="color: #50FA7B">-h</span>, <span style="color: #50FA7B">--help</span>                       Print help information
    <span style="color: #50FA7B">-l</span>, <span style="color: #50FA7B">--level</span> <span style="color: #50FA7B">&lt;LEVEL&gt;</span>              A level of verbosity, and can be used multiple times (default:
                                     info) [possible values: debug, info, warn, error]
        <span style="color: #50FA7B">--log-dir</span> <span style="color: #50FA7B">&lt;LOG_DIR&gt;</span>          Directory to store logs in (default: current directory)
        <span style="color: #50FA7B">--system-env</span> <span style="color: #50FA7B">&lt;SYSTEM_ENV&gt;</span>    Whether to use system environment variables (default: true)
    <span style="color: #50FA7B">-V</span>, <span style="color: #50FA7B">--version</span>                    Print version information
</pre>

## Job Syntax
The job file is a YAML file that describes the job to be run. It is heavily inspired by [Github Actions Workflow](https://docs.github.com/en/actions/learn-github-actions/workflow-syntax-for-github-actions) files, but contains some differences. Documentation is therefore provided in a similar fashion with `job` as root key (referring to the job file itself).

### Jobs

#### `job.id`
The job id is a string that uniquely identifies the job. It is used to identify the job in the logs. By default, it is set to the name of the job file.

#### `job.name`
The job name is a string that is used to display the job in the logs or other output. By default, it is set to the name of the job file.

#### `job.env`
The job env is a list of environment variables that will be set before the job is run. They are also used for each job.

```yaml
env:
  FOO: bar
  BAZ: qux
```

#### `job.cwd`
The job cwd is a string that is used to set the current working directory before the job is run. All the other relative paths used in the job are relative to this directory.

### Tasks

#### `job.tasks.<task>.id`
The task id is a string that uniquely identifies the task. It is used to identify the task in the logs. By default, it is set as transformed task name or command (run) name.

#### `job.tasks.<task>.name`
The task name is a string that is used to display the task in the logs or other output.

#### `job.tasks.<task>.env`
The task env is a list of environment variables that will be set before the task is run. They are also used for the task and merged with all the other env variables.

```yaml
tasks:
  - name: run
    env:
      FOO: bar
      BAZ: qux
```

#### `job.tasks.<task>.cwd`
The task cwd is a string that is used to set the current working directory before the task is run. All the other relative paths used in the task are relative to this directory.

#### `job.tasks.<task>.run`
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

#### `job.tasks.<task>.shell`
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


#### `job.tasks.<task>.shell_path`
The shell path is a string that is used to specify the path to the shell to use for the tasks. If not specified, the shell is determined by the ones available in the system.

### Hooks

#### `job.hooks`
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

#### `job.tasks.<task>.hooks`
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

### Logging

#### `job.logging.<log>.type`
The log type is a string that is used to specify the type of the log. It is one of the following:

* `console` - Log to the console.
* `file` - Log to a file.

#### `job.logging.<log>.name`
The logging name is a string that is used to display the logging option in the logs or other output.

#### `job.logging.<log>.stdout`
If set to `true`, the standard output of the task will be captured and logged.

Default: `true`

#### `job.logging.<log>.stderr`
If set to `true`, the standard error of the task will be captured and logged.

Default: `true`

#### `job.logging.<log>.hooks`
If set to `true`, the standard output and error of the hook tasks will also be captured and logged.

Default: `true`

#### `job.logging.<log>.internal`
If set to `true`, the log output of `nauman` will also be captured and logged.

Default: `true`

#### File logging
#### `job.logging.<log>.file`
Refers to the file path of the file to store the log into.

If `split` is set to `true`, this file should refer to a directory. The log will be stored in a file named after the task id within this directory.

If a relative path is given, then it is relative to the log directory.

#### `job.logging.<log>.split`
If set to `true`, the log will be stored in a file named after the task id within the specified directory.

#### Console logging
none

### Global Options
#### `job.options.shell`
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

#### `job.options.shell_path`
The shell path is a string that is used to specify the path to the shell to use for the tasks. If not specified, the shell is determined by the ones available in the system.

#### `job.options.dry_run`
If set to `true`, the job will always execute in dry run mode.

#### `job.options.ansi`
If set to `false`, the job will not output ANSI escape codes.

#### `job.options.log_level`
The log level is a string that is used to specify the log level. It is one of the following:

* `debug` - Debug level.
* `info` - Info level.
* `warn` - Warn level.
* `error` - Error level.

#### `job.options.log_dir`
The log directory is a string that is used to specify the directory to store the logs. If not specified, the logs will be stored in the current working directory.

#### `job.options.system_env`
If set to `true`, the job will use the system environment variables. If set to `false`, the job will only use the environment variables explicitly defined in the job, task or in the cli.

### Execution Policies

#### `job.policy`
The job policy is the global execution policy enforced for all the tasks unless overridden. It is a string that can be one of the following:

* `always` - Always execute the task regardless of prior task status.
* `prior_success` - Execute the task only if prior task has succeeded.
* `no_prior_failed` - Execute the task only if no other task has failed.

#### `job.tasks.<task>.policy`
The task policy is the execution policy enforced for the task. It is a string that can be one of the following:

* `always` - Always execute the task regardless of prior task status.
* `prior_success` - Execute the task only if prior task has succeeded.
* `no_prior_failed` - Execute the task only if no other task has failed.

## Alternatives
If this is not what you are looking for, check out these cool alternatives:
* Bash or Makefile
* [just](https://github.com/casey/just) - is a handy way to save and run project-specific commands
* [fastlane](https://github.com/fastlane/fastlane) - is a tool for iOS and Android developers to automate tedious tasks like generating screenshots, dealing with provisioning profiles, and releasing your application
* [Apache Airflow](https://airflow.apache.org/) - is a platform created by the community to programmatically author, schedule and monitor workflows.

## TODO
* [ ] Add support for .env files
* [ ] Add more tests
* [ ] Add a way to natively run web requests
* [ ] Add a way to write outputs of different tasks
* [ ] Add a templating system
* [ ] Add a way to specify per log whether ansi is enabled or not

## Contributing

As this is a hobby project, contributions are very welcome!

The easiest way for you to contribute right now is to use nauman, and see where it's lacking. 

If you have a use case nauman does not cover, please file an issue. This is immensely useful to me, to anyone wanting to contribute to the project, and to you as well if the feature is implemented.

If you're interested in helping fix an [existing issue](https://github.com/EgorDm/nauman/issues), or an issue you just filed, help is appreciated.

See [CONTRIBUTING](./CONTRIBUTING.md) for technical information on contributing.


## License

This project is licensed under the terms of the MIT license. See the [LICENSE](LICENSE) file.
