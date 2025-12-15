#include "ProcessManager.h"
#include <iostream>
#include <cstring>
#include <memory>

#ifdef WIN32
#include <windows.h>
#include <process.h>
#endif

#ifdef __APPLE__
#include <unistd.h>
#include <sys/wait.h>
#endif

#ifdef __linux__
#include <unistd.h>
#include <sys/wait.h>
#endif

// ProcessManager implementation
uint32_t ProcessManager::spawn_process(const char* cmd, uint32_t cmd_len, const char** args, uint32_t arg_count) {
    try {
        std::string command = join_command(cmd, cmd_len, args, arg_count);

#ifdef WIN32
        STARTUPINFOA si = { sizeof(si) };
        PROCESS_INFORMATION pi;

        if (!CreateProcessA(
            nullptr,
            const_cast<char*>(command.c_str()),
            nullptr,
            nullptr,
            FALSE,
            0,
            nullptr,
            nullptr,
            &si,
            &pi
        )) {
            std::cerr << "Failed to create process: " << GetLastError() << std::endl;
            return 0;
        }

        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
        return pi.dwProcessId;
#endif

#ifdef __APPLE__
        pid_t pid = fork();
        if (pid == 0) {
            // Child process
            std::vector<char*> argv;
            argv.push_back(strdup(cmd));
            for (uint32_t i = 0; i < arg_count; ++i) {
                argv.push_back(strdup(args[i]));
            }
            argv.push_back(nullptr);

            execvp(cmd, argv.data());

            // If we get here, exec failed
            std::cerr << "Failed to execute: " << cmd << std::endl;
            exit(1);
        } else if (pid < 0) {
            std::cerr << "Fork failed" << std::endl;
            return 0;
        }

        return static_cast<uint32_t>(pid);
#endif

#ifdef __linux__
        pid_t pid = fork();
        if (pid == 0) {
            // Child process
            std::vector<char*> argv;
            argv.push_back(strdup(cmd));
            for (uint32_t i = 0; i < arg_count; ++i) {
                argv.push_back(strdup(args[i]));
            }
            argv.push_back(nullptr);

            execvp(cmd, argv.data());

            // If we get here, exec failed
            std::cerr << "Failed to execute: " << cmd << std::endl;
            exit(1);
        } else if (pid < 0) {
            std::cerr << "Fork failed" << std::endl;
            return 0;
        }

        return static_cast<uint32_t>(pid);
#endif

    } catch (const std::exception& e) {
        std::cerr << "Process spawn error: " << e.what() << std::endl;
        return 0;
    }
}

bool ProcessManager::kill_process(uint32_t pid) {
#ifdef WIN32
    HANDLE hProcess = OpenProcess(PROCESS_TERMINATE, FALSE, pid);
    if (hProcess == nullptr) {
        return false;
    }

    bool result = TerminateProcess(hProcess, 1) != 0;
    CloseHandle(hProcess);
    return result;
#endif

#ifdef __APPLE__
    return kill(static_cast<pid_t>(pid), SIGTERM) == 0;
#endif

#ifdef __linux__
    return kill(static_cast<pid_t>(pid), SIGTERM) == 0;
#endif
}

int32_t ProcessManager::wait_process(uint32_t pid) {
#ifdef WIN32
    HANDLE hProcess = OpenProcess(SYNCHRONIZE | PROCESS_QUERY_INFORMATION, FALSE, pid);
    if (hProcess == nullptr) {
        return -1;
    }

    DWORD result = WaitForSingleObject(hProcess, INFINITE);
    CloseHandle(hProcess);

    if (result == WAIT_OBJECT_0) {
        DWORD exit_code;
        if (GetExitCodeProcess(hProcess, &exit_code)) {
            return static_cast<int32_t>(exit_code);
        }
    }

    return -1;
#endif

#ifdef __APPLE__
    int status;
    if (waitpid(static_cast<pid_t>(pid), &status, 0) == -1) {
        return -1;
    }

    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    } else if (WIFSIGNALED(status)) {
        return -WTERMSIG(status);
    }

    return -1;
#endif

#ifdef __linux__
    int status;
    if (waitpid(static_cast<pid_t>(pid), &status, 0) == -1) {
        return -1;
    }

    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    } else if (WIFSIGNALED(status)) {
        return -WTERMSIG(status);
    }

    return -1;
#endif
}

std::string ProcessManager::join_command(const char* cmd, uint32_t cmd_len, const char** args, uint32_t arg_count) {
    std::string command(cmd, cmd_len);

    for (uint32_t i = 0; i < arg_count; ++i) {
        command += " ";
        command += args[i];
    }

    return command;
}

// FFI interface implementations
extern "C" {

uint32_t spawn_process(const char* cmd, uint32_t cmd_len, const char** args, uint32_t arg_count) {
    return ProcessManager::spawn_process(cmd, cmd_len, args, arg_count);
}

bool kill_process(uint32_t pid) {
    return ProcessManager::kill_process(pid);
}

int32_t wait_process(uint32_t pid) {
    return ProcessManager::wait_process(pid);
}

}