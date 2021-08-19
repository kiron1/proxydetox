#ifndef PROXYDETOX_H
#define PROXYDETOX_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

struct ProxydetoxServer;

struct ProxydetoxServer *proxydetox_new(char const *pac_script, bool negotiate,
                                        uint16_t port);

void proxydetox_run(struct ProxydetoxServer *server);

void proxydetox_shutdown(struct ProxydetoxServer *server);

void proxydetox_drop(struct ProxydetoxServer *server);

#ifdef __cplusplus
}
#endif

#endif