FROM rust:1.71.1
WORKDIR /var/facts-server
COPY . .
RUN make test
RUN make release 
RUN cargo install --path .
EXPOSE 8888
CMD ["rust-test-server"]
