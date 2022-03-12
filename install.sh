#!/bin/bash
if [ -f /etc/influxdb/influxdb.conf ]; then
    echo "InfluxDB already installed, let's clean up its old files"
    sudo systemctl stop influxdb
    sudo systemctl disable influxdb
    sudo apt-get remove -y influxdb
    sudo apt-get purge -y influxdb
    sudo rm -rf /etc/influxdb
    sudo rm -rf /var/lib/influxdb
    sudo rm -rf /opt/db
fi
echo "Installing InfluxDB 1.8.x"

sudo bash -c 'wget -qO- https://repos.influxdata.com/influxdb.key | sudo apt-key add -'
export DISTRIB_ID=$(lsb_release -si); export DISTRIB_CODENAME=$(lsb_release -sc)
sudo bash -c "echo \"deb https://repos.influxdata.com/${DISTRIB_ID,,} ${DISTRIB_CODENAME} stable\" > /etc/apt/sources.list.d/influxdb.list"
sudo apt-get update>/dev/null
sudo apt-get install -y influxdb
sudo systemctl unmask influxdb.service
sudo mkdir -p /opt/db /opt/db/data /opt/db/meta /opt/influx /opt/log
sudo chown -R influxdb:influxdb /opt/db /opt/db/data /opt/db/meta /opt/influx /opt/log
if [ -f /etc/influxdb/influxdb_backup.conf ]; then
    sudo cp /etc/influxdb/influxdb_backup.conf /etc/influxdb/influxdb.conf
else
    sudo cp /etc/influxdb/influxdb.conf /etc/influxdb/influxdb_backup.conf
fi

sudo sed -i '244i \ \ access-log-path = \"/opt/log/access.log\"' /etc/influxdb/influxdb.conf
sudo sed -i 's+\ \ dir\ =\ \"/var/lib/influxdb/meta\"+\ \ dir\ =\ \"/opt/db/meta\"+g' /etc/influxdb/influxdb.conf
sudo sed -i 's+\ \ dir\ =\ \"/var/lib/influxdb/data\"+\ \ dir\ =\ \"/opt/db/data\"+g' /etc/influxdb/influxdb.conf
sudo sed -i 's+\ \ dir\ =\ \"/var/lib/influxdb/wal\"+\ \ dir\ =\ \"/opt/influx/wal\"+g' /etc/influxdb/influxdb.conf
sudo systemctl restart influxdb
echo "wait for 5 seconds before create database"
sleep 5

echo "Creating database:thingworx"
influx -execute "create database thingworx"

echo "Creating user:twadmin with password."
influx -execute "CREATE USER twadmin WITH PASSWORD 'twadmin' with ALL PRIVILEGES"

sudo sed -i 's+#\ auth-enabled\ =\ false+auth-enabled\ =\ true+g' /etc/influxdb/influxdb.conf

echo "enable authentication in InfluxDB"
sudo systemctl restart influxdb


echo "Installing Grafana..."
GrafanaService="/usr/lib/systemd/system/grafana-server.service"
if [ ! -f "${GrafanaService}" ]; then
    echo "It seems like first time install grafana"
    sudo apt-get install -y apt-transport-https
    sudo apt-get install -y software-properties-common wget
    wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
    echo "deb https://packages.grafana.com/enterprise/deb stable main" | sudo tee -a /etc/apt/sources.list.d/grafana.list
    sudo apt-get update
else
    sudo systemctl stop grafana-server
    sudo apt-get remove -y grafana-enterprise
    sudo mv /etc/grafana/grafana.ini /etc/grafana/grafana.ini.backup
fi
sudo apt-get install -y grafana-enterprise
sudo systemctl daemon-reload
sudo systemctl enable grafana-server
sudo bash -c 'systemctl restart grafana-server'

echo "Installing tsample..."
if [ -f /etc/systemd/system/tsample.service ]; then
    echo "tsample already installed, let's clean up it first."
    sudo systemctl stop tsample
    sudo systemctl disable tsample
    sudo rm /etc/systemd/system/tsample.service
    if [ -f /etc/tsample ]; then
        sudo rm -rf /etc/tsample
        echo "Existing tsample config file removed, please make sure to reconfig it."
    fi
fi
wget https://github.com/xudesheng/tsample/releases/download/v4.1.0/tsample-x86_64-linux-musl.tar.gz
tar -xzf tsample-x86_64-linux-musl.tar.gz
echo "install tsample to /usr/local/bin"
sudo mv tsample /usr/local/bin/tsample
sudo chmod +x /usr/local/bin/tsample
sudo mkdir /etc/tsample
echo "Exporting tsample config file to /etc/tsample/tsample.conf"
sudo /usr/local/bin/tsample -c /etc/tsample/tsample.yaml -e

MYGROUP=tsample
MYUSER=tsample
if grep -q $MYGROUP /etc/group
then
    echo "group:${MYGROUP} exists"
else
    echo "create group:${MYGROUP}"
    sudo addgroup --system --quiet --force-badname ${MYGROUP}
fi

if id "$MYUSER" &>/dev/null; then
    echo user:$MYUSER exists
else
    echo "create user:${MYUSER}"
    sudo adduser --system --home /home/${MYUSER}/ --ingroup ${MYGROUP} --disabled-password --shell /bin/false --force-badname ${MYUSER}
fi
sudo chown -R ${MYUSER}:${MYGROUP} /etc/tsample

sudo tee /etc/systemd/system/tsample.service>/dev/null<<EOT
[Unit]
Description=tsample service
Wants=network-online.target
After=network-online.target

[Service]
User=${MYUSER}
Group=${MYGROUP}
Type=simple
ExecStart=/usr/local/bin/tsample -c /etc/tsample/tsample.yaml

[Install]
WantedBy=multi-user.target
EOT
sudo systemctl daemon-reload

echo "Installation complete. It's time for you to configure tsample."
echo "Please modify /etc/tsample/tsample.yaml to your needs before you use `sudo systemctl start tsample` to start the service."
echo "Specially you need to modify the following fields:"
echo " - line 104: server_name, it likely should be 127.0.0.1"
echo " - line 47/49/51: your thingworx connection url."
echo " - line 56: app_key for remote access."
echo " and then `sudo systemctl enable tsample` and `sudo systemctl start tsample` to start the service."
