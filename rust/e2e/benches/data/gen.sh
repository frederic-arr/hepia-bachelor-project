if [ ! -f "talosconfig.yaml" ]; then
    talosctl gen config test https://10.2.0.15:6443 --with-docs=false --with-cluster-discovery=false --with-examples=false --with-kubespan=false
    mv talosconfig talosconfig.yaml
    mv controlplane.yaml talos.yaml
    rm worker.yaml
fi
