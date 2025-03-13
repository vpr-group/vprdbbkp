#!/bin/bash
# Script to extract pg_dump binaries and dependencies from PostgreSQL Docker images

# Configuration
OUTPUT_DIR="pg_utils"
PG_VERSIONS=(9.6 10 11 12 13 14 15 16)

# Create output directory
mkdir -p "$OUTPUT_DIR"
echo "Created output directory: $OUTPUT_DIR"

# Function to extract PostgreSQL utilities and dependencies
extract_pg_utils() {
  local version=$1
  echo "Processing PostgreSQL $version..."
  
  # Create version-specific directories
  mkdir -p "$OUTPUT_DIR/$version/bin"
  mkdir -p "$OUTPUT_DIR/$version/lib"
  
  # Pull the Docker image
  echo "Pulling postgres:$version image..."
  docker pull postgres:$version
  
  # Start a container from the image
  echo "Starting temporary container..."
  container_id=$(docker run -d postgres:$version sleep 30)
  
  # Extract the binaries
  echo "Extracting binaries..."
  docker cp $container_id:/usr/lib/postgresql/$version/bin/pg_dump "$OUTPUT_DIR/$version/bin/"
  docker cp $container_id:/usr/lib/postgresql/$version/bin/pg_restore "$OUTPUT_DIR/$version/bin/"
  docker cp $container_id:/usr/lib/postgresql/$version/bin/psql "$OUTPUT_DIR/$version/bin/"
  
  # Extract dependencies
  echo "Extracting dependencies..."
  docker exec $container_id bash -c "mkdir -p /tmp/deps"
  
  for binary in pg_dump pg_restore psql; do
    echo "Finding dependencies for $binary..."
    
    # Create a temporary script inside the container
    docker exec $container_id bash -c "cat > /tmp/find_deps.sh << 'EOF'
#!/bin/bash
BINARY=\"/usr/lib/postgresql/$version/bin/\$1\"
mkdir -p /tmp/deps
ldd \$BINARY | grep -v \"=>\" | awk '{print \$1}' | grep -v \"linux-vdso.so\" > /tmp/deps/deps.txt
ldd \$BINARY | grep \"=>\" | awk '{print \$3}' | grep -v \"not found\" >> /tmp/deps/deps.txt
EOF"
    
    # Make script executable and run it
    docker exec $container_id bash -c "chmod +x /tmp/find_deps.sh && /tmp/find_deps.sh $binary"
    
    # Copy the dependencies list
    docker cp $container_id:/tmp/deps/deps.txt "$OUTPUT_DIR/$version/deps_$binary.txt"
    
    # Copy each dependency
    while read dep; do
      # Skip virtual libraries and standard libraries
      if [[ "$dep" == *"linux-vdso"* || "$dep" == *"/lib64/ld-linux-x86-64.so"* ]]; then
        continue
      fi
      echo "Copying dependency: $dep"
      docker cp $container_id:$dep "$OUTPUT_DIR/$version/lib/" || echo "Warning: Could not copy $dep"
    done < "$OUTPUT_DIR/$version/deps_$binary.txt"
    
    # Remove the temporary deps file
    rm -f "$OUTPUT_DIR/$version/deps_$binary.txt"
  done
  
  # Stop and remove the container
  docker stop $container_id > /dev/null
  docker rm $container_id > /dev/null
  
  # Make the binaries executable
  chmod +x "$OUTPUT_DIR/$version/bin/"*
  
  # Create wrapper scripts
  for binary in pg_dump pg_restore psql; do
    cat > "$OUTPUT_DIR/$version/bin/${binary}_wrapper" << EOF
#!/bin/bash
DIR="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
export LD_LIBRARY_PATH="\$DIR/../lib:\$LD_LIBRARY_PATH"
exec "\$DIR/$binary" "\$@"
EOF
    chmod +x "$OUTPUT_DIR/$version/bin/${binary}_wrapper"
  done
  
  echo "PostgreSQL $version utilities extracted successfully."
  echo "---------------------------------------------------"
}

# Main loop to process each version
for version in "${PG_VERSIONS[@]}"; do
  extract_pg_utils $version
done

echo "All PostgreSQL utilities have been extracted to $OUTPUT_DIR"
echo "Directory structure:"
find "$OUTPUT_DIR" -type d | sort

# Add this at the end of your script, before the final echo statements
echo "Fixing permissions..."
if [ $(id -u) -ne 0 ]; then
  # We're not running as root, so use sudo
  sudo chown -R $(whoami):$(whoami) "$OUTPUT_DIR"
else
  # We're already root, so just set permissions
  chown -R $(whoami):$(whoami) "$OUTPUT_DIR"
fi

echo "Done!"