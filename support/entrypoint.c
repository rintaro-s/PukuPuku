#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>

int main(int argc, char *argv[]) {
    // Get the ARGV0 and APPDIR environment variables
    const char *argv0_env = getenv("ARGV0");
    const char *appdir_env = getenv("APPDIR");

    // Check if APPDIR is set
    if (!appdir_env) {
        fprintf(stderr, "Error: APPDIR environment variable not set.\n");
        return 1;
    }

    // Buffer to hold the full path of the binary
    char binary_path[1024];

    // Try the ARGV0 binary first if it's set
    if (argv0_env) {
        snprintf(binary_path, sizeof(binary_path), "%s/usr/bin/%s", appdir_env, argv0_env);
    } else {
        // If ARGV0 is not set, fallback to missioncenter
        snprintf(binary_path, sizeof(binary_path), "%s/usr/bin/missioncenter", appdir_env);
    }

    // Check if the binary exists
    if (access(binary_path, X_OK) != 0) {
        // If the ARGV0 binary doesn't exist, fallback to missioncenter
        snprintf(binary_path, sizeof(binary_path), "%s/usr/bin/missioncenter", appdir_env);
    }

    // Prepare arguments for exec
    char *new_argv[argc + 1];  // Array to hold arguments for the new process
    new_argv[0] = binary_path;  // The binary path is the new argv[0]
    for (int i = 1; i < argc; i++) {
        new_argv[i] = argv[i];  // Pass along all the arguments except argv[0]
    }
    new_argv[argc] = NULL;  // Null-terminate the array

    // Execute the binary
    execv(new_argv[0], new_argv);

    // If execv returns, there was an error
    perror("Error executing binary");
    return 1;
}
