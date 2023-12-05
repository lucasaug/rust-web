#!/bin/bash

read INDATA
printf "Content-Type: text/html\n\n"
printf "Body: ${INDATA}<br />"
export VAR=$(env)
printf "${VAR//$'\n'/<br />}"

