#include <openssl/sha.h>
#include <caml/mlvalues.h>
#include <caml/memory.h>
#include <caml/alloc.h>
CAMLprim value hex_analysis_sha256(value bytes) {
  CAMLparam1(bytes);
  unsigned char digest[SHA256_DIGEST_LENGTH];
  char hex[SHA256_DIGEST_LENGTH * 2];
  static const char digits[] = "0123456789abcdef";
  SHA256((const unsigned char *)String_val(bytes), caml_string_length(bytes), digest);
  for (int i = 0; i < SHA256_DIGEST_LENGTH; ++i) {
    hex[2*i] = digits[digest[i] >> 4];
    hex[2*i+1] = digits[digest[i] & 15];
  }
  CAMLreturn(caml_alloc_initialized_string(sizeof hex, hex));
}
