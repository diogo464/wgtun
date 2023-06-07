# wgtun

A WireGuard TCP tunnel.

## The problem

Trying to use WireGuard from behind a restrictive firewall can be a problem since alot of times UDP is blocked.
This requires workarounds like having to share a phone's mobile internet over a hostspot.

## The solution

A possible solution to this problem is to simply encapsulate all WireGuard traffic inside a TCP connection and use one of the unrestricted ports to bypass the UDP block.
As stated in https://www.wireguard.com/known-limitations/ this is intentionally not supported because of the bad performance of tunneling TCP-over-TCP.
I don't know how much the performance deteriorates but from experience it is still good enough to run ssh without problems.

## How to use wgtun

On the client run `wgtun client --server example.com:51820`. This will bind socket on localhost and forward any UDP packets over TCP to the server. The WireGuard configuration on the client has to be modified to use `localhost:51820` as the endpoint.

On the server, or any machine that can reach the server, run `wgtun server --target localhost:51820` where `localhost:51820` is where the WireGuard server is listening.

### Container

A container image is available at `ghcr.io/diogo464/wgtun`.
