FROM helikon/subvt-backend-lib:0.1.3 as builder

FROM helikon/subvt-backend-base:0.1.3
# copy executable
COPY --from=builder /subvt/bin/subvt-network-status-updater /usr/local/bin/
CMD ["subvt-network-status-updater"]