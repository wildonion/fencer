#!/bin/bash
cd ..
sudo rm .env && sudo mv .env.prod .env
sudo apt update -y && sudo apt upgrade && sudo apt install -y libpq-dev pkg-config build-essential libudev-dev libssl-dev librust-openssl-dev

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
# cargo install diesel_cli --no-default-features --features postgres
cargo install sqlx-cli --no-default-features --features native-tls,postgres
cargo install sea-orm-cli

sudo apt install -y protobuf-compiler libssl-dev zlib1g-dev
wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo dpkg -i libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo apt install -y snapd && sudo snap install core; sudo snap refresh core
sudo snap install --classic certbot && sudo ln -s /snap/bin/certbot /usr/bin/certbot
cargo install sqlant && sudo apt install -y openjdk-11-jdk && sudo apt install -y graphviz

sqlant postgresql://postgres:$PASSWORD@localhost/rustacki > $(pwd)/infra/rustacki.uml
java -jar $(pwd)/infra/plantuml.jar $(pwd)/infra/rustacki.uml

jobs="jobs/*"
for f in $jobs
do
    crontab $f
done  
crontab -u root -l 


git clone https://github.com/cossacklabs/themis.git
cd themis
make
sudo make install

sudo apt-get install -y apache2-utils
echo \n"-> Creating Nginx Logs Dir Password"\n\t;
htpasswd -c infra/docker/nginx/.htpasswd rustacki


echo "-ˋˏ✄┈┈┈┈ "
echo \n"╰┈➤ make sure you have all the ssl cert and config files related to your app domain inside `infra/docker/nginx` folder! [y/n]?"\n
echo "-ˋˏ✄┈┈┈┈ "
read CERTCOMPLETED

if [[ $CERTCOMPLETED == "Y" || $CERTCOMPLETED == "y" ]]; then
    sudo docker compose up -d --no-cache
else
    echo \t"run me again once you get done with adding cert and config files"
fi