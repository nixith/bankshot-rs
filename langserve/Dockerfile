FROM python:3.11-slim

# installs
RUN apt-get update && \
    apt-get install --no-install-recommends -y python3 && \
    apt-get install --no-install-recommends -y python3-pip && \
    apt-get install --no-install-recommends -y python3-venv

# Non root user
RUN useradd -m app

# in app
WORKDIR /app
COPY ./local_langserve.py ./requirements.txt /app
RUN chown -R app:app /app

# switch user
USER app

# Enable a py env
RUN python3 -m venv .venv
ENV PATH="/app/.venv/bin:$PATH"

# pip installs
RUN pip install -r requirements.txt

#run langserve
CMD python3 local_langserve.py
