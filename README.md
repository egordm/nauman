<!-- markdownlint-disable -->
<div id="top"></div>
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
  <a href="https://github.com/EgorDm/nauman/blob/master/JOB_SYNTAX.md">Job Syntax</a>
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

<p align="right">(<a href="#top">back to top</a>)</p>

## Examples
For more examples, see the [examples](examples) directory.
* [Using Hooks](#using-hooks)
* [Logging](#logging)
* [Using Environment Variables](#using-environment-variables)
* [More](https://github.com/EgorDm/nauman/tree/master/examples)

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

<p align="right">(<a href="#top">back to top</a>)</p>

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

<p align="right">(<a href="#top">back to top</a>)</p>

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

<p align="right">(<a href="#top">back to top</a>)</p>

## Features
* [Hook everything](#hook-everything)
* [Flexible Logging](#flexible-logging)
* [Context variables](#context-variables)
* [Configurable task plan](#configurable-task-plan)
* [Different shell types](#different-shell-types)
* [Dry run](#dry-run)
* [Task Outputs](#task-outputs)
* [Multiline commands](#multiline-commands)
* [Dotenv files](#dotenv-files)
* [Change your working directory](#change-your-working-directory)

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

<p align="right">(<a href="#top">back to top</a>)</p>

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

<p align="right">(<a href="#top">back to top</a>)</p>

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

<p align="right">(<a href="#top">back to top</a>)</p>

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

<p align="right">(<a href="#top">back to top</a>)</p>

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

<p align="right">(<a href="#top">back to top</a>)</p>

### Dry run
Want to make sure that your job is configured correctly? You can run your job in dry run mode. This will verify that all tasks are syntactically correct, all shells are usable and warn you about any potential issues (such as missing directories).

```shell
nauman --dry-run my_job.yml
```

<p align="right">(<a href="#top">back to top</a>)</p>

### Task Outputs
During the execution of every task, a temporary file is created where you can store the output variables. These files are automatically deleted after the task is finished. The variables specified in the output files will be loaded into the global context as environment variables.

The output file accepts dotenv style syntax.

```yaml
tasks:
  ...
  - name: Append output to the output file
    run: echo "foo=bar"  >> "$NAUMAN_OUTPUT_FILE"
  - name: Use the output variable
    run: echo $foo
```

<p align="right">(<a href="#top">back to top</a>)</p>

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

<p align="right">(<a href="#top">back to top</a>)</p>

### Dotenv files
You can use dotenv files to define variables for your tasks.

```yaml
options:
  dotenv: /path/to/my_env.env
```

<p align="right">(<a href="#top">back to top</a>)</p>

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

<p align="right">(<a href="#top">back to top</a>)</p>

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

<p align="right">(<a href="#top">back to top</a>)</p>

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

<p align="right">(<a href="#top">back to top</a>)</p>

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

<p align="right">(<a href="#top">back to top</a>)</p>

## [Job Syntax](https://github.com/EgorDm/nauman/blob/master/JOB_SYNTAX.md)

## Alternatives
If this is not what you are looking for, check out these cool alternatives:
* Bash or Makefile
* [just](https://github.com/casey/just) - is a handy way to save and run project-specific commands
* [fastlane](https://github.com/fastlane/fastlane) - is a tool for iOS and Android developers to automate tedious tasks like generating screenshots, dealing with provisioning profiles, and releasing your application
* [Apache Airflow](https://airflow.apache.org/) - is a platform created by the community to programmatically author, schedule and monitor workflows.

<p align="right">(<a href="#top">back to top</a>)</p>

## TODO
* [x] Add support for .env files
* [ ] Add more tests
* [ ] Add a way to natively run web requests
* [x] Add a way to write outputs of different tasks
* [ ] Add a templating system
* [ ] Add a way to specify per log whether ansi is enabled or not
* [ ] Add flock support
* [ ] Always add console logging (only specify whether stdout and stderr should be logged)

## Contributing

As this is a hobby project, contributions are very welcome!

The easiest way for you to contribute right now is to use nauman, and see where it's lacking. 

If you have a use case nauman does not cover, please file an issue. This is immensely useful to me, to anyone wanting to contribute to the project, and to you as well if the feature is implemented.

If you're interested in helping fix an [existing issue](https://github.com/EgorDm/nauman/issues), or an issue you just filed, help is appreciated.

See [CONTRIBUTING](./CONTRIBUTING.md) for technical information on contributing.

<p align="right">(<a href="#top">back to top</a>)</p>

## License

This project is licensed under the terms of the MIT license. See the [LICENSE](LICENSE) file.

<p align="right">(<a href="#top">back to top</a>)</p>
