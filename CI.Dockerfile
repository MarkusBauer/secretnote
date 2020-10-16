FROM scratch
COPY fe /fe/
COPY secretnote /secretnote
ENV SECRETNOTE_BIND=0.0.0.0:8080 SECRETNOTE_REDIS=redis
ENTRYPOINT ["/secretnote"]
