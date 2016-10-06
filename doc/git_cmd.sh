 git branch
git add ./
git commit -m "MethGeno works"
git push origin dev


git checkout master
git merge dev
git push origin master
=======================================================================
First, clone a remote Git repository and cd into it:

$ git clone git://example.com/myproject
$ cd myproject
Next, look at the local branches in your repository:

$ git branch
* master
But there are other branches hiding in your repository! You can see these using the -a flag:

$ git branch -a
* master
  remotes/origin/HEAD
  remotes/origin/master
  remotes/origin/v1.0-stable
  remotes/origin/experimental
If you just want to take a quick peek at an upstream branch, you can check it out directly:

$ git checkout origin/experimental
But if you want to work on that branch, you'll need to create a local tracking branch:

$ git checkout -b experimental origin/experimental
and you will see

Branch experimental set up to track remote branch experimental from origin.
Switched to a new branch 'experimental'
That last line throw some people "New branch" - huh? What it really means is a new local branch that gets the branch from the index and creates it locally for you. The previous line is actually more informative as it tells you that the branch is being set up to track the remote branch, which usually means the origin/branch_name branch

Now, if you look at your local branches, this is what you'll see:

$ git branch
* experimental
  master
You can actually track more than one remote repository using git remote.

$ git remote add win32 git://example.com/users/joe/myproject-win32-port
$ git branch -a
* master
  remotes/origin/HEAD
  remotes/origin/master
  remotes/origin/v1.0-stable
  remotes/origin/experimental
  remotes/win32/master
  remotes/win32/new-widgets
At this point, things are getting pretty crazy, so run gitk to see what's going on:

$ gitk --all &
#=============================================================================================


Maybe you just need to commit. I ran into this when I did:

mkdir repo && cd repo
git remote add origin /path/to/origin.git
git add .
Oops! Never committed!

git push -u origin master
error: src refspec master does not match any.
All I had to do was:

git commit -m 'initial commit'
git push origin master
Success!
#==========================================
  420  git branch dev
  423  git commit -m "add branch dev"
  424  git push origin master
  425  git branch -d devlop
  426  git commit -m "add branch dev"
  427  git branch
  428  git checkout 
  430  git checkout dev
  440  git remove -r lib/SUBCMD/
  441  git rmgit rm lib/SUBCMD/
  442  git rm -r4 lib/SUBCMD/
  443  git rm -r lib/SUBCMD/
  446  git commit -m "rm lib/SUBCMD/"
  447  git push origin dev
  847  git branch
  850  vi ../git_cmd.sh
  851  git add ./
  852  vi ../git_cmd.sh
  853  git commit -m "MethGeno works"
  854  git commit -m "MethGeno works.Tested. "
  855  git push origin dev
  856  vi ../git_cmd.sh 
  857  git merge
  858  git checkout -b master
  859  git checkout  master
  860  git merge
  861  git merge dev
  863  git push origin master
  864  git merge dev
  865  git push origin master
  866  git pull --rebase origin master
  872  git push origin master
  875  git check dev
  876  git checkout dev
  898  git branch
 1037  history |grep 'git'
 1038  history |grep 'git' >> ../git_cmd.sh 
