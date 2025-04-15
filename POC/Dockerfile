FROM python:3.12-slim-bullseye

ADD https://github.com/krallin/tini/releases/download/v0.19.0/tini /tini
RUN chmod +x /tini

# Install any python dependencies
COPY requirements.txt requirements.txt
RUN pip3 install --no-cache-dir -r requirements.txt

# Install any additional local binaries
COPY --chmod=555 ./bin/* /usr/local/bin/

ENTRYPOINT ["/tini", "-g", "--", "/bin/bash", "-c"]