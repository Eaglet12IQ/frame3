FROM python:3.9-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    postgresql-client ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/legacy
COPY legacy.py ./
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt
COPY run.sh /usr/local/bin/run_legacy.sh
RUN sed -i 's/\r$//' /usr/local/bin/run_legacy.sh && chmod +x /usr/local/bin/run_legacy.sh

CMD ["/usr/local/bin/run_legacy.sh"]
