import sys
import yaml
import json

with open("config/config.yaml", "r") as f:
    data = yaml.safe_load(f)
    with open("target/config.yaml", "w") as fout:
        yaml.dump(data, fout)
    with open("target/config.json", "w") as fout:
        json.dump(data, fout, indent=4)

    servers = data.get("thingworx_servers")
    if servers:
        if len(servers) > 0:
            print("subsystems:{}".format(servers[0].get("subsystems")))
