ARG version
FROM helikon/subvt-backend-lib:$version as builder

FROM helikon/subvt-backend-base:$version
# copy executable
COPY --from=builder /subvt/bin/subvt-validator-details-server /usr/local/bin/
CMD ["subvt-validator-details-server"]