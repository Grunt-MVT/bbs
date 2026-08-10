#include <napi.h>

#include <cstdint>
#include <cstring>
#include <string>
#include <vector>

#include "bbs_ffi.h"

namespace {

Napi::Error StatusError(Napi::Env env, int32_t status) {
  const char *message = bbs_status_message(status);
  if (message == nullptr) {
    message = "unknown status";
  }
  return Napi::Error::New(
      env, std::string("libbbsplus: ") + message + " (status " +
               std::to_string(status) + ")");
}

BbsByteSlice ByteSliceFromBuffer(const Napi::Buffer<uint8_t> &buffer) {
  BbsByteSlice slice;
  slice.data = buffer.Data();
  slice.len = buffer.Length();
  return slice;
}

BbsByteSlice ByteSliceFromString(const std::string &value) {
  BbsByteSlice slice;
  slice.data = value.empty()
                   ? nullptr
                   : reinterpret_cast<const uint8_t *>(value.data());
  slice.len = value.size();
  return slice;
}

std::string BufferToString(BbsByteBuffer buffer) {
  std::string value;
  if (buffer.data != nullptr && buffer.len > 0) {
    value.assign(reinterpret_cast<const char *>(buffer.data), buffer.len);
  }
  bbs_free_buffer(buffer);
  return value;
}

Napi::Value PidOrder(const Napi::CallbackInfo &info) {
  Napi::Env env = info.Env();
  BbsStringArray raw = bbs_pid_order();
  if (raw.data == nullptr) {
    StatusError(env, BBS_ERROR_NULL_POINTER).ThrowAsJavaScriptException();
    return env.Null();
  }

  Napi::Array values = Napi::Array::New(env, raw.len);
  for (size_t i = 0; i < raw.len; i++) {
    if (raw.data[i] == nullptr) {
      StatusError(env, BBS_ERROR_NULL_POINTER).ThrowAsJavaScriptException();
      return env.Null();
    }
    values.Set(i, Napi::String::New(env, raw.data[i]));
  }
  return values;
}

Napi::Value CanonicalString(const Napi::CallbackInfo &info) {
  Napi::Env env = info.Env();
  if (info.Length() < 1 || !info[0].IsString()) {
    Napi::TypeError::New(env, "expected a string").ThrowAsJavaScriptException();
    return env.Null();
  }

  std::string input = info[0].As<Napi::String>().Utf8Value();
  BbsByteBuffer out{nullptr, 0};
  int32_t status = bbs_canonical_string(ByteSliceFromString(input), &out);
  if (status != BBS_OK) {
    bbs_free_buffer(out);
    StatusError(env, status).ThrowAsJavaScriptException();
    return env.Null();
  }
  return Napi::String::New(env, BufferToString(out));
}

Napi::Value CanonicalNationality(const Napi::CallbackInfo &info) {
  Napi::Env env = info.Env();
  if (info.Length() < 1 || !info[0].IsString()) {
    Napi::TypeError::New(env, "expected a string").ThrowAsJavaScriptException();
    return env.Null();
  }

  std::string input = info[0].As<Napi::String>().Utf8Value();
  BbsByteBuffer out{nullptr, 0};
  int32_t status = bbs_canonical_nationality(ByteSliceFromString(input), &out);
  if (status != BBS_OK) {
    bbs_free_buffer(out);
    StatusError(env, status).ThrowAsJavaScriptException();
    return env.Null();
  }
  return Napi::String::New(env, BufferToString(out));
}

Napi::Value CanonicalNationalityList(const Napi::CallbackInfo &info) {
  Napi::Env env = info.Env();
  if (info.Length() < 1 || !info[0].IsArray()) {
    Napi::TypeError::New(env, "expected a string array")
        .ThrowAsJavaScriptException();
    return env.Null();
  }

  Napi::Array input = info[0].As<Napi::Array>();
  std::vector<std::string> owned;
  owned.reserve(input.Length());
  std::vector<const char *> pointers;
  pointers.reserve(input.Length());

  for (uint32_t i = 0; i < input.Length(); i++) {
    Napi::Value item = input.Get(i);
    if (!item.IsString()) {
      Napi::TypeError::New(env, "expected a string array")
          .ThrowAsJavaScriptException();
      return env.Null();
    }
    owned.push_back(item.As<Napi::String>().Utf8Value());
  }
  for (const std::string &value : owned) {
    pointers.push_back(value.c_str());
  }

  BbsByteBuffer out{nullptr, 0};
  int32_t status = bbs_canonical_nationality_list(
      pointers.data(), pointers.size(), &out);
  if (status != BBS_OK) {
    bbs_free_buffer(out);
    StatusError(env, status).ThrowAsJavaScriptException();
    return env.Null();
  }
  return Napi::String::New(env, BufferToString(out));
}

Napi::Value VerifyProof(const Napi::CallbackInfo &info) {
  Napi::Env env = info.Env();
  if (info.Length() < 3 || !info[0].IsBuffer() || !info[1].IsBuffer() ||
      !info[2].IsArray()) {
    Napi::TypeError::New(env, "expected (Buffer, Buffer, RevealedMessage[])")
        .ThrowAsJavaScriptException();
    return env.Null();
  }

  Napi::Buffer<uint8_t> public_key = info[0].As<Napi::Buffer<uint8_t>>();
  Napi::Buffer<uint8_t> proof = info[1].As<Napi::Buffer<uint8_t>>();
  Napi::Array revealed = info[2].As<Napi::Array>();

  std::vector<BbsIndexedMessage> indexed;
  indexed.reserve(revealed.Length());
  // Keep Buffer references alive for the duration of the C call.
  std::vector<Napi::Buffer<uint8_t>> buffers;
  buffers.reserve(revealed.Length());

  for (uint32_t i = 0; i < revealed.Length(); i++) {
    Napi::Value item = revealed.Get(i);
    if (!item.IsObject()) {
      Napi::TypeError::New(env, "revealed message must be an object")
          .ThrowAsJavaScriptException();
      return env.Null();
    }
    Napi::Object object = item.As<Napi::Object>();
    Napi::Value index_value = object.Get("index");
    Napi::Value data_value = object.Get("data");
    if (!index_value.IsNumber() || !data_value.IsBuffer()) {
      Napi::TypeError::New(env, "revealed message needs index and data Buffer")
          .ThrowAsJavaScriptException();
      return env.Null();
    }

    buffers.push_back(data_value.As<Napi::Buffer<uint8_t>>());
    BbsIndexedMessage message;
    message.index = index_value.As<Napi::Number>().Uint32Value();
    message.data = buffers.back().Data();
    message.len = buffers.back().Length();
    indexed.push_back(message);
  }

  int32_t status = bbs_verify_proof(ByteSliceFromBuffer(public_key),
                                    ByteSliceFromBuffer(proof), indexed.data(),
                                    indexed.size());
  if (status == BBS_OK) {
    return Napi::Boolean::New(env, true);
  }
  if (status == BBS_ERROR_VERIFY_FAILED) {
    return Napi::Boolean::New(env, false);
  }
  StatusError(env, status).ThrowAsJavaScriptException();
  return env.Null();
}

Napi::Object Init(Napi::Env env, Napi::Object exports) {
  exports.Set("pidOrder", Napi::Function::New(env, PidOrder));
  exports.Set("canonicalString", Napi::Function::New(env, CanonicalString));
  exports.Set("canonicalNationality",
              Napi::Function::New(env, CanonicalNationality));
  exports.Set("canonicalNationalityList",
              Napi::Function::New(env, CanonicalNationalityList));
  exports.Set("verifyProof", Napi::Function::New(env, VerifyProof));
  return exports;
}

} // namespace

NODE_API_MODULE(bbsplus_node, Init)
