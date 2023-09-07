FROM rust:1.67 as builder
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
COPY --from=builder /cargo/bin/myapp /bin/myapp
CMD ["aws-cfn-stack-outputs"]