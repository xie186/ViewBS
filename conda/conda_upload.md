# How to Setup Automatic Uploads to Anaconda from Travis CI in 15 minutes
**TL;DR:** Edit `.travis.yaml` to install Anaconda and to run `conda_upload.sh` after testing. Edit `meta.yaml` to take in the environmental variables `$VERSION` and `$CONDA_BLD_PATH`. Create `conda_upload.sh` which sets the needed environmental variables, builds the tar archive, and uploads it to Anaconda. Finally edit some stuff on your Anaconda and Travis CI account so they can talk.

## Intro
The following steps will detail how to automatically trigger Anaconda builds and uploads from Travis CI. This will only upload successful builds in the master branch and if there are multiple commits in a single day, it'll only keep the latest one. Both of these settings can easily be changed.

## Edit .travis.yaml
First, edit `.travis.yml` so that it installs Anaconda.
```
install:
  # Install Anaconda
  - if [[ "$TRAVIS_PYTHON_VERSION" == "2.7" ]]; then
      wget https://repo.continuum.io/miniconda/Miniconda2-latest-Linux-x86_64.sh -O miniconda.sh;
    else
      wget https://repo.continuum.io/miniconda/Miniconda3-latest-Linux-x86_64.sh -O miniconda.sh;
    fi
  - bash miniconda.sh -b -p $HOME/miniconda
  - export PATH="$HOME/miniconda/bin:$PATH"
  - hash -r
  - conda config --set always_yes yes --set changeps1 no
  - conda update -q conda
```
The code below will allow for Travis to install  `conda-build` and the `anaconda-client` after sucessful tests so we can upload to Anaconda. This allows Travis to use `conda-build` to automatically create a tar archive and `anaconda upload` to upload it. 
Also the code below runs the `conda_upload.sh` script, which is detailed below, to run after all of the tests pass on Travis CI and only on the master branch. **Don't forget to change the path to the conda folder in the command below to match your directory structure.** You can change `after_success` to `after_script` so that it'll always upload to Anaconda, regardless of if the tests pass.

```
after_success:
  - test $TRAVIS_BRANCH = "master" && conda install conda-build && conda install anaconda-client && bash conda/conda_upload.sh
```

## Edit meta.yaml
Now edit `meta.yaml` with the below code so that the version can change based on the `$VERSION` environmental variable. `$VERSION` is set to the current date in `conda_upload.sh` detailed later.

```
package:
  version: {{ environ['VERSION'] }}
```

Also add the below code to `meta.yaml` to allow the use of the `$VERSION` and `$CONDA_BLD_PATH` environmental variables when executing `conda build`. We need `$CONDA_BLD_PATH` to customize the `conda build` location so that we can easily access the created tar archive when uploading.

```
build:
  script_env:
   - VERSION
   - CONDA_BLD_PATH
```
**Also make sure that there isn't a `git_rev` value in `meta.yml`, as this will cause conda to use that git tag instead of the latest code in your git repository.**

## Creating conda_upload.sh

In your conda directory with `meta.yaml` and `build.sh`, create the following file titled `conda_upload.sh`.
```
# Only need to change these two variables
PKG_NAME=cdp
USER=zshaheen

OS=$TRAVIS_OS_NAME-64
mkdir ~/conda-bld
conda config --set anaconda_upload no
export CONDA_BLD_PATH=~/conda-bld
export VERSION=`date +%Y.%m.%d`
conda build .
anaconda -t $CONDA_UPLOAD_TOKEN upload -u $USER -l nightly $CONDA_BLD_PATH/$OS/$PKG_NAME-`date +%Y.%m.%d`-0.tar.bz2 --force
```
Based on your project, you'll need to change `PKG_NAME`, and `USER`. Python testing on OSX with Travis is very slow currently, so be aware of this. You can set `OS=linux-64` to just run on Linux.

This script, which again is called when the tests pass on Travis, runs `conda build` to create a tar titled using the `PKG_NAME` and current date. Then `anaconda upload` is called to upload this tar archive to Anaconda under `$USER` with the `nightly` label. `--force` overwrites any existing tar archive on Anaconda with the same name.

## Giving Travis access to upload to Anaconda
You might have noticed that `$CONDA_UPLOAD_TOKEN` in `conda_upload.sh` was not set. This token is used to authenticate the `anaconda upload` command. Let's now get this token and securely store in on Travis. **If not stored securely, anyone with this token can easily delete everything that user has stored on Anaconda.**

There is a fully command line procedure to get the token, but that isn't covered here.

1. Login to the account you want Travis to use to upload on [anaconda.org](https://anaconda.org).
2. Click on your username on the top left and go to 'My Settings'.
3. On the left hand panel, go to 'Access' and enter your password as requested.
4. Now we'll create an API token. Give it a name, and **check at least both 'Allow read access to the API site' and 'Allow write access to the API site'**.
5. Create the token and copy it.
6. Login to your account on [travis-ci.org](https://travis-ci.org) and go to the repository that you want to add this automatic functionality to.
7. On the right next to 'More options' go to 'Settings' in the hamburger menu.
8. Add an environment variable with the name `CONDA_UPLOAD_TOKEN` and give it the value of the API token that you copied from [anaconda.org](https://anaconda.org).

## Conclusion
Now add `meta.yaml`, `.travis.yaml`, and newly created `conda_upload.sh`. Then commit and push to your git repository and you should soon see a newly create file under your project's Anaconda page.
