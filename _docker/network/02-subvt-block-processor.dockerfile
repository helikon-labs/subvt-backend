FROM helikon/subvt-backend-lib:0.1.5 as builder

FROM helikon/subvt-backend-base:0.1.5
# copy executable
COPY --from=builder /subvt/bin/subvt-block-processor /usr/local/bin/
CMD ["subvt-block-processor"]