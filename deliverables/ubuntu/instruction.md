sudo mv tsample.service /etc/systemd/system/tsample.service
sudo systemctl daemon-reload
sudo systemctl enable tsample.service
sudo systemctl start tsample.service
