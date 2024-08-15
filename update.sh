#!/bin/bash
REPOs=(sel4_common sel4_task sel4_ipc sel4_vspace sel4_cspace kernel driver-collect serial-impl/pl011 serial-impl/sbi serial-frame)
PARENT_COMMIT_ID=$(git log -1 --pretty=%H | head -n 1)
echo $PARENT_COMMIT_ID
cd ..

# urls=("git@github.com:rel4team/sel4_common.git"
#     "git@github.com:rel4team/sel4_task.git"
#     "git@github.com:rel4team/sel4_ipc.git "
#     "git@github.com:rel4team/sel4_vspace.git"
#     "git@github.com:rel4team/sel4_cspace.git"
#     "git@github.com:rel4team/rel4_kernel.git"
#     "git@github.com:rel4team/driver-collect.git"
#     "git@github.com:rel4team/serial-impl-pl011.git"
#     "git@github.com:rel4team/serial-impl-sbi.git"
#     "git@github.com:rel4team/serial-frame.git"
# )

# for url in ${urls[@]}; do
#     git clone $url
# done

for repo in ${REPOs[@]}; do
    # cd $repo
    # COMMIT_ID=$(git log -1 --pretty=%H | head -n 1)
    # echo $COMMIT_ID
    PWD=$(pwd)
    cd mi-dev-integral-rel4/$repo
    sed -i "10c\ \tparent = $PARENT_COMMIT_ID" .gitrepo
    cd $PWD
done
