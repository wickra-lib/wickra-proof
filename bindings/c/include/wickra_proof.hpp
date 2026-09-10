// Optional C++ convenience layer over the wickra-proof C ABI (`wickra_proof.h`).
//
// The C ABI hands out a raw handle that must be released exactly once with
// `wickra_proof_free`, and its command entry point writes into a caller-owned
// buffer and returns the length it wanted. Both are easy to get subtly wrong in
// C++, and neither needs to be: this wraps the handle in a move-only RAII owner
// and the command in a std::string round trip that grows the buffer once if the
// first attempt was too small.
//
//     #include "wickra_proof.hpp"
//
//     wickra_proof::Prover p;
//     auto out = p.command(R"({"cmd":"version"})");
//
// Header-only, and adds no runtime cost beyond the C calls themselves.

#ifndef WICKRA_PROOF_HPP
#define WICKRA_PROOF_HPP

#include "wickra_proof.h"

#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace wickra_proof {

/// Move-only owner of a `WickraProof *`. Frees exactly once, at scope exit.
class Prover {
 public:
  Prover() : handle_(wickra_proof_new()) {
    if (handle_ == nullptr) {
      throw std::runtime_error("wickra_proof_new returned null");
    }
  }

  ~Prover() {
    if (handle_ != nullptr) {
      wickra_proof_free(handle_);
    }
  }

  Prover(const Prover &) = delete;
  Prover &operator=(const Prover &) = delete;

  Prover(Prover &&other) noexcept : handle_(other.handle_) {
    other.handle_ = nullptr;
  }

  Prover &operator=(Prover &&other) noexcept {
    if (this != &other) {
      if (handle_ != nullptr) {
        wickra_proof_free(handle_);
      }
      handle_ = other.handle_;
      other.handle_ = nullptr;
    }
    return *this;
  }

  /// The raw handle, for calling the C ABI directly. Ownership stays here.
  WickraProof *get() const noexcept { return handle_; }

  /// Send one command envelope and return the response.
  ///
  /// The C entry point returns the length it needed. A negative value is an
  /// error code from the header (WICKRA_PROOF_ERR_*); a value larger than the
  /// buffer means the response did not fit, so it is called again with a
  /// buffer of exactly that size. Two attempts at most: the second is told
  /// the true length rather than guessing.
  std::string command(const std::string &cmd_json) const {
    std::vector<char> buf(4096);
    int32_t needed = wickra_proof_command(handle_, cmd_json.c_str(), buf.data(), buf.size());
    if (needed < 0) {
      throw std::runtime_error("wickra_proof_command failed with code " + std::to_string(needed));
    }
    if (static_cast<size_t>(needed) >= buf.size()) {
      buf.assign(static_cast<size_t>(needed) + 1, '\0');
      needed = wickra_proof_command(handle_, cmd_json.c_str(), buf.data(), buf.size());
      if (needed < 0) {
        throw std::runtime_error("wickra_proof_command failed with code " + std::to_string(needed));
      }
    }
    return std::string(buf.data(), static_cast<size_t>(needed));
  }

 private:
  WickraProof *handle_;
};

/// The library version, as reported by the C ABI.
inline std::string version() { return std::string(wickra_proof_version()); }

}  // namespace wickra_proof

#endif  // WICKRA_PROOF_HPP
