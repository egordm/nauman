<!-- markdownlint-disable -->
<div align="center">
    <h1>NauMan</h1>
    <p>
        <b>A CI inspired approach for local job automation.</b>
    </p>
    <p>
    </p>
    <br/>
</div>
<!-- markdownlint-enable -->


## About

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

## Advanced Examples

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

## FAQ

## Installation
The binary name for nauman is `nauman`.

[Archives of precompiled binaries for nauman are available for Windows, macOS and Linux. Linux and Windows binaries are static executables.](https://github.com/BurntSushi/EgorDm/nauman) Users of platforms not explicitly mentioned below are advised to download one of these archives.

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

## Job Syntax

### Jobs

### Tasks

### Hooks

### Logging

### Global Options

### Execution Policies

## Contributing

As this is a hobby project, contributions are very welcome!

The easiest way for you to contribute right now is to use nauman, and see where it's lacking. 

If you have a use case nauman does not cover, please file an issue. This is immensely useful to me, to anyone wanting to contribute to the project, and to you as well if the feature is implemented.

If you're interested in helping fix an [existing issue](https://github.com/EgorDm/nauman/issues), or an issue you just filed, help is appreciated.

See [CONTRIBUTING](./CONTRIBUTING.md) for technical information on contributing.


## License

This project is licensed under the terms of the MIT license. See the [LICENSE](LICENSE) file.