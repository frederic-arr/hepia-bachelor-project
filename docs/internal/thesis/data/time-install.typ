#import "lib.typ": *

#import "../lib.typ": *
#import "../../packages.typ": *
#import packages.lilaq as lq

#let (
    time_to_config,
    time_to_install,
    time_to_kernel,
    time_to_init,
    time_to_supervisor,
    time_to_reconcile,
    time_to_dhcp,
    time_to_downloading_image,
    time_to_download_image,
    time_to_run_container,
    time_to_kernel_post,
    time_to_init_post,
    time_to_supervisor_post,
    time_to_reconcile_post,
    time_to_dhcp_post,
    time_to_run_container_post,
) = lq.load-txt(read("time-install.csv"))

#let time_to_config = time_to_config.map(n => n / 1000)
#let time_to_install = time_to_install.map(n => n / 1000)
#let time_to_kernel = time_to_kernel.map(n => n / 1000)
#let time_to_init = time_to_init.map(n => n / 1000)
#let time_to_supervisor = time_to_supervisor.map(n => n / 1000)
#let time_to_reconcile = time_to_reconcile.map(n => n / 1000)
#let time_to_dhcp = time_to_dhcp.map(n => n / 1000)
#let time_to_downloading_image = time_to_downloading_image.map(n => n / 1000)
#let time_to_download_image = time_to_download_image.map(n => n / 1000)
#let time_to_run_container = time_to_run_container.map(n => n / 1000)
#let time_to_kernel_post = time_to_kernel_post.map(n => n / 1000)
#let time_to_init_post = time_to_init_post.map(n => n / 1000)
#let time_to_supervisor_post = time_to_supervisor_post.map(n => n / 1000)
#let time_to_reconcile_post = time_to_reconcile_post.map(n => n / 1000)
#let time_to_dhcp_post = time_to_dhcp_post.map(n => n / 1000)
#let time_to_run_container_post = time_to_run_container_post.map(n => n / 1000)
