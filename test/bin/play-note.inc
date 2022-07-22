
play_note() {
    if [ "${IFACE}" == "" ] ; then
       echo 'must set env variable PP_IFACE'
    fi
    echo "playing ${1} on IFACE '${IFACE}'"
    : ${ON:=0.40}
    : ${OFF:=0.10}
    : ${VEL:=4F}
    amidi -p ${IFACE} -S "90 ${1} ${VEL}"
    sleep ${ON}
    amidi -p ${IFACE} -S "80 ${1} 7F"
    sleep ${OFF}
}
