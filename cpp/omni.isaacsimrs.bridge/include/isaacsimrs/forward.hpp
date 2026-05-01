// SPDX-License-Identifier: MPL-2.0
#pragma once

#include "isaac-sim-bridge/src/lib.rs.h"
#include <string>

namespace isaacsimrs::detail
{

inline rust::Str str_from(const std::string& s)
{
    return rust::Str{ s.data(), s.size() };
}

template <typename T, typename Container>
inline rust::Slice<const T> slice_from(const Container& c)
{
    return rust::Slice<const T>{ c.data(), c.size() };
}

}
