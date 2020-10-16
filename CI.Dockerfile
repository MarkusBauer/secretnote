FROM scratch
COPY fe /fe/
COPY secretnote /secretnote
ENTRYPOINT ["/secretnote"]
