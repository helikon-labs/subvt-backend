ARG version
FROM helikon/subvt-backend-lib:$version as builder

FROM helikon/subvt-backend-base:$version
# copy executable
COPY --from=builder /subvt/bin/subvt-validator-list-updater /usr/local/bin/
CMD ["subvt-validator-list-updater"]