#!/bin/bash

read INDATA
printf "Content-Type: text/html\n\n"
printf "Stdin data: ${INDATA}<br />"
export VAR=$(env)
printf "${VAR//$'\n'/<br />}"

