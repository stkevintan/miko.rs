# Multi-stage build leveraging pre-built binaries
FROM scratch AS binary-selector
ARG TARGETOS=linux
ARG TARGETARCH=amd64

# Copy all built binaries from the build context
COPY bin/ /bin/

# Final stage - minimal runtime image
FROM alpine:latest

# Create non-root user for security
RUN adduser --disabled-password --gecos "" miko

WORKDIR /app

# Copy the appropriate binary based on target architecture
ARG TARGETOS
ARG TARGETARCH
# We map Docker's TARGETARCH to our binary naming convention if necessary
# In our build script we use amd64 and arm64
COPY --from=binary-selector /bin/miko-${TARGETOS}-${TARGETARCH}* ./miko

# Create data directory, set binary as executable and change ownership
RUN mkdir -p /app/data && chmod +x ./miko && \
  chown miko:miko /app/data ./miko

# Switch to non-root user
USER miko

# Set default environment variables
ENV PORT=8081
ENV DATABASE_URL=sqlite:///app/data/miko.db

# Expose port
EXPOSE 8081

# Command to run
CMD ["./miko"]
