FROM helikon/subvt-backend-lib:0.1.5 as builder

FROM helikon/subvt-backend-base:0.1.5
# copy executable
COPY --from=builder /subvt/bin/subvt-validator-list-server /usr/local/bin/
CMD ["subvt-validator-list-server", "--inactive"]