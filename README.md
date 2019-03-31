# ARP Microservice
A simple web API to do an [arp](https://en.wikipedia.org/wiki/Address_Resolution_Protocol) scan on the local network and get human-readable results of vendor hardware on the local wifi network.

## What it does
* Request comes to API
* API gets list of existing IP addresses on local network
* API sends ARP request to each IP address and receives MAC addresses
* API matches each MAC address against list in app state
* API calls macvendors.com for each MAC that is found that is not in app state
* Results from macvendors are added to app state
* Response is returned to requester