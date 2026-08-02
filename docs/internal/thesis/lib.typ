#import "lib/figures.typ": *
#import "lib/helpers.typ": *
#import "lib/todo.typ": *

// #let made-by-self = [réalisé par Frédéric ARROYO]
#let made-by-self = none

#let repo(path, ..body) = {
    let base_path = "flg_bachelors/tb/2026/container-infrastructure-deployment-os"
    let tag = "v0.0.0-dev.3.doc"
    link(
        "https://gitedu.hesge.ch/" + base_path + "/-/tree/" + tag + "/" + path,
        if body.len() == 0 {
            [#path]
        } else {
            body.at(0)
        },
    )
}
