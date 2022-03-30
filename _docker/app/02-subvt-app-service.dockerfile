FROM helikon/subvt-backend-lib:0.1.3 as builder

FROM helikon/subvt-backend-base:0.1.3
# copy executable
COPY --from=builder /subvt/bin/subvt-app-service /usr/local/bin/
CMD ["subvt-app-service"]