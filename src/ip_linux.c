#include <arpa/inet.h>
#include <sys/socket.h>
#include <ifaddrs.h>
#include <stdlib.h>

/**
 * Retrieves a list of local addresses upto max_addrs and stores them into a.
 *
 * @return number of of retrieved addresses
 */
int get_local_addr(int *a, size_t max_addrs)
{
    struct ifaddrs     *addrs;
    struct ifaddrs     *next;
    struct sockaddr_in *addr;

    getifaddrs(&addrs);
    int i = 0;
    for (next = addrs; next; next = next->ifa_next) {
        if (i >= max_addrs)
            break;

        if (next->ifa_addr->sa_family == AF_INET) {
            addr = (struct sockaddr_in*) next->ifa_addr;
            a[i] = addr->sin_addr.s_addr;
            i++;
        }
    }
    freeifaddrs(addrs);
    return i;
}
