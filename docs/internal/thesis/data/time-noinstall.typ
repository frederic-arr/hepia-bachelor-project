#import "lib.typ": *

#import "../lib.typ": *
#import "../../packages.typ": *
#import packages.lilaq as lq

#let (
    time_to_kernel,
    time_to_init,
    time_to_supervisor,
    time_to_reconcile,
    time_to_dhcp,
    time_to_downloading_image,
    time_to_download_image,
    time_to_run_container,
) = lq.load-txt(read("time-noinstall.csv"))
