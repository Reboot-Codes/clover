FROM python:alpine

WORKDIR /opt/app

COPY . .

RUN python3 -m venv ./venv
RUN source ./venv/bin/activate
RUN python3 -m pip install -r requirements.txt

CMD ["python3" "main.py"]
