

a GPS location consumer over RMQ, it consumes incoming location events from the relevant exchange then it checks each point (lat, lon) against the defined geofence (supports both **Polygon** and **LineString**) for that device based on the imei. See postman collection to define geofence for a device.

[this](https://github.com/wildonion/gps-location-consumer) service stores each location event coming from the exchange inside the db along with their cumulative mileage.

## How 2 setup, develop, and deploy?

### Dev env

#### step0) create database 

> make sure you've created the database using:

```bash
sqlx database create
```

#### step1) create migration folder (make sure you have already!)

> make sure you uncomment the runtime setup inside its `Cargo.toml` file.

```bash
sea-orm-cli migrate init -d migration
```
#### step2) create migration files

> make sure you've installed the `sea-orm-cli` then create migration file per each table operation, contains `Migrations` structure with `up` and `down` methods extended by the `MigrationTrait` interface, take not that you must create separate migration per each db operation when you're in production.

```bash
sea-orm-cli migrate generate "table_name"
```

#### step3) apply pending migrations (fresh, refresh, reset, up or down)

> once you've done with adding changes in your migration files just run the following to apply them in db.

```bash
# rollback all applied migrations, then reapply all migrations
sea-orm-cli migrate refresh # or up
```

#### step4) generate entity files for ORM operations

> generate Rust structures from your applied migrations inside the db for the `rustackigeo` database in `entity/src` folder after that you can proceed with editing each eintity like adding hooks for different actions on an active model.

```bash
# generate entities for the database and all tables
sea-orm-cli generate entity -u postgres://postgres:geDteDd0Ltg2135FJYQ6rjNYHYkGQa70@localhost/geodb -o src/entities --with-serde both --serde-skip-deserializing-primary-key
# generate entity for an sepecific table only, eg: generating entity for hoops table
sea-orm-cli generate entity -t hoops -o src/entities --with-serde both --serde-skip-deserializing-primary-key
# don't skip deserializing primary key
sea-orm-cli generate entity -u postgres://postgres:geDteDd0Ltg2135FJYQ6rjNYHYkGQa70@localhost/geodb -o src/entities --with-serde both
```
#### step4) run server

> when you run server with `--fresh` command it'll fresh all migrations at startup (drop all tables from the database, then reapply all migrations) otherwise it'll only apply migrations (calling `up` method of all migration files).

```bash
# -------------------------------
# ------ rustackigeo server --------
# -------------------------------
# launch as http with freshing db
cargo run --bin rustackigeo -- --server http --fresh # default is http and fresh migrations
# or see help
cargo run --bin rustackigeo -- --help
```

### Prod env

> make sure you've opened all necesary domains inside your DNS panel per each nginx config file and changed the `your-app.app` to your own domain name in every where mostly the nginx config files and the `APP_NAME` in `consts.rs`.

```bash
# -----------------------
# ---- read/write access
sudo chown -R root:root . && sudo chmod -R 777 . 
```

#### 🚀 the CI/CD approach:

> this approach can be used if you need a fully automatic deployment process, it uses github actions to build and publish all images on a self-hosted docker registry on a custom VPS, so update the github ci/cd workflow files inside `.github/workflows` folder to match your VPS infos eventually on every push the ci/cd process will begin to building and pushing automatically the docker images to the self-hosted registry. instead of using a custom registry you can use either ducker hub or github packages! it's notable that you should renew nginx service everytime you add a new domain or subdomain (do this on adding a new domain), `./renew.sh` script creates ssl certificates with certbot for your new domain and add it inside the `infra/docker/nginx` folder so nginx docker can copy them into its own container! for every new domain there must be its ssl certs and nginx config file inside that folder so make sure you've setup all your domains before pushing to the repo. continue reading... 

##### me before you! (make sure you've done followings properly before pushing to your repo):

- **step1)** first thing as the first, connect your device to github for workflow actions using `gh auth login -s workflow` so you can push easily.

- **step2)** run `sudo rm .env && sudo mv .env.prod .env` then update necessary variables inside `.env` file.

- **step3)** the docker [registry](https://distribution.github.io/distribution/) service is up and running on your VPS.

- **step4)** you would probably want to make `logs` dir and `docker.sprun.ir` routes secure and safe, you can achive this by adding an auth gaurd on the docker registry subdomain and the logs dir inside their nginx config files eventually setup the password for `logs` dir and `docker.sprun.ir` route by running `sudo apt-get install -y apache2-utils && htpasswd -c infra/docker/nginx/.htpasswd rustackigeo` command, the current one is `rustackigeo@1234`.

- **step5)** setup `DOCKER_PASSWORD`, `DOCKER_USERNAME`, `SERVER_HOST`, `SERVER_USER` and `SERVER_PASSWORD` secrets and variables on your repository.

- **step6)** setup nginx config and ssl cert files per each domain and subdomain using `renew.sh` script then put them inside `infra/docker/nginx` folder, **you MUST do this before you get pushed to the repo on github cause there is already an nginx container inside the `docker-compose.yml`** so make sure you are running the script in your VPS and you've setup all domains and subdomains already in your DNS panel pointing to your VPS ip address.

- **step7)** created a `/root/hoopoe` folder on your VPS containing the `docker-compose.yml` file only and update its path inside the `cicd.yml` file, take this note that the default location and directory for none root users are `/home`.

- **step8)** each image name inside your compose file must be prefixed with your docker hub registry endpoint which in this case is `docker.sprun.ir` cause the doamin is already pointing to the docker registry hosted on `localhost:5000` on VPS.

##### What's happening inside the `cicd.yml` file?

- **step1)** read the codes inside the repository to find the `docker-compose.yml` file.

- **step1)** try to login (docker username and password) to your custom docker hub (the registry on your VPS secured with nginx auth gaurd).

- **step2)** build all docker container images inside your `docker-compose.yml` file.

- **step3)** eventually it push them to your custom docker hub registry.

- **step4)** ssh to the VPS and cd to where you've put the `docker-compose.yml` file in there then pull and up all pushed docker containers from the VPS hub inside the VPS.