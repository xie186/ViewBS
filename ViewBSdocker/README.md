# Docker

* Understanding where images come from
* Pulling a Docker image from Docker Hub
* Pushing a Docker image to Docker Hub

## Getting an image to Docker Hub

Imagine you made your own Docker image and would like to share it with the world you can sign up for an account on https://hub.docker.com/. After verifying your email you are ready to go and upload your first docker image.

Log in on https://hub.docker.com/
Click on Create Repository.
Choose a name (e.g. verse_gapminder) and a description for your repository and click Create.

```
docker login --username=xie186
password: 1*****
```

```
docker images
#REPOSITORY              TAG       IMAGE ID         CREATED           SIZE
#verse_gapminder_gsl     latest    023ab91c6291     3 minutes ago     1.975 GB
#verse_gapminder         latest    bb38976d03cf     13 minutes ago    1.955 GB
#rocker/verse            latest    0168d115f220     3 days ago        1.954 GB
```

```
docker tag 851afd2bd80b xie186/viewbs:firsttry
docker push xie186/viewbs
## ${PWD}:/data: means to mount current directory under /data in the container
docker run -v ${PWD}:/data -w /data   940ed43a2cb8 bitmapperBS --index test.fa
```


## Use `singularity` with existing `Docker` images

### First install `singularity`

You can either ask your administrator to install or install it yourself. 

Here is the instruction to install it on Ubuntu: https://singularity.lbl.gov/install-linux#adding-the-mirror-and-installing

### Use `singularity` to pull docker image
```
singularity pull docker://xie186/viewbs
singularity exec viewbs.simg ViewBS --help
```

Here is a screenshot for what will be printed on the terminal: 
![](https://user-images.githubusercontent.com/20909751/54481412-a06ece80-480a-11e9-9952-efb44d390d08.png)


## Reference

https://ropenscilabs.github.io/r-docker-tutorial/04-Dockerhub.html

https://www.osc.edu/resources/getting_started/howto/howto_use_docker_and_singularity_containers_at_osc

https://hpc.nih.gov/apps/singularity.html



