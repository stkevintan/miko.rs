# Multi-stage build leveraging pre-built binaries
FROM scratch AS binary-selector
ARG TARGETOS=linux
ARG TARGETARCH=amd64

# Copy all built binaries from the build context
COPY bin/ /bin/

# Final stage - minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies (ca-certificates is needed for HTTPS)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN adduser --disabled-password --gecos "" miko

WORKDIR /app

# Copy the appropriate binary based on target architecture
ARG TARGETOS
ARG TARGETARCH
# We map Docker's TARGETARCH to our binary naming convention if necessary
# In our build script we use amd64 and arm64
COPY --from=binary-selector /bin/miko-rs-${TARGETOS}-${TARGETARCH}* ./miko-rs

# Set binary as executable and change ownership
RUN chmod +x ./miko-rs && chown miko:miko ./miko-rs

# Switch to non-root user
USER miko

# Set default environment variables
ENV PORT=8081
ENV DATABASE_URL=sqlite:///app/data/miko.db

# Expose port
EXPOSE 8081

# Command to run
CMD ["./miko-rs"]
