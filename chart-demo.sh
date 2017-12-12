#!/bin/bash

options="--start now-240s --end now --slope-mode --lower-limit 0"

for item in /tmp/*.cpu.rrd; do
    echo "Generating chart for '$item'"
    rrdtool graph $item.png \
        $options \
        --title "Demo CPU chart: $item" \
        --watermark "University of Westminster" \
        --vertical-label "% usage" \
        DEF:user=$item:user:MAX \
        DEF:system=$item:system:MAX \
        DEF:iowait=$item:iowait:MAX \
        DEF:other=$item:other:MAX \
        AREA:user#00FF00:"user":STACK \
        AREA:system#0000FF:"system":STACK \
        AREA:iowait#FF0000:"iowait":STACK \
        AREA:other#000000:"other":STACK
done

for item in /tmp/*.memory.rrd; do
    echo "Generating chart for '$item'"
    rrdtool graph $item.png \
        $options \
        --title "Demo RAM chart: $item" \
        --watermark "University of Westminster" \
        --base 1024 \
        --vertical-label "memory usage" \
        DEF:u=$item:used:MAX \
        DEF:b=$item:buffers:MAX \
        DEF:c=$item:cache:MAX \
        DEF:f=$item:free:MAX \
        CDEF:used=u,1024,* \
        CDEF:buffers=b,1024,* \
        CDEF:cache=c,1024,* \
        CDEF:free=f,1024,* \
        AREA:used#00FF00:"used":STACK \
        AREA:buffers#0000FF:"buffers":STACK \
        AREA:free#00FFFF:"free":STACK \
        AREA:cache#FF00FF:"cache"
done

