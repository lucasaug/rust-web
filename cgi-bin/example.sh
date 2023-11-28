#!/bin/bash

read INDATA
echo "Content-Type: text/html\n\n"
echo "Stdin data: ${INDATA}<br />"
export VAR=$(env)
echo "${VAR//$'\n'/<br />}"

