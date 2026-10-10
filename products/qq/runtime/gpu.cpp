#include "gpu.h"

#include <QByteArray>
#include <QtGlobal>

#ifdef Q_OS_WIN
#define NOMINMAX
#include <dxgi1_6.h>
#include <windows.h>
#endif

namespace qq {

QString preferHighPerformanceGpu()
{
#ifdef Q_OS_WIN
    if (qEnvironmentVariableIsSet("QT_D3D_ADAPTER_INDEX"))
        return {};
    IDXGIFactory6 *factory = nullptr;
    if (FAILED(CreateDXGIFactory1(__uuidof(IDXGIFactory6), reinterpret_cast<void **>(&factory))))
        return {};
    QString chosen;
    IDXGIAdapter1 *fastest = nullptr;
    if (SUCCEEDED(factory->EnumAdapterByGpuPreference(0, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE, __uuidof(IDXGIAdapter1),
                                                      reinterpret_cast<void **>(&fastest)))) {
        DXGI_ADAPTER_DESC1 want{};
        fastest->GetDesc1(&want);
        fastest->Release();
        // Qt counts adapters in the system's own order (EnumAdapters1): find the fastest's place in it.
        IDXGIAdapter1 *adapter = nullptr;
        for (UINT i = 0; factory->EnumAdapters1(i, &adapter) != DXGI_ERROR_NOT_FOUND; ++i) {
            DXGI_ADAPTER_DESC1 desc{};
            adapter->GetDesc1(&desc);
            adapter->Release();
            if (desc.AdapterLuid.LowPart == want.AdapterLuid.LowPart && desc.AdapterLuid.HighPart == want.AdapterLuid.HighPart) {
                if (i != 0) {
                    qputenv("QT_D3D_ADAPTER_INDEX", QByteArray::number(i));
                    chosen = QString::fromWCharArray(desc.Description);
                }
                break;
            }
        }
    }
    factory->Release();
    return chosen;
#else
    return {};
#endif
}

}  // namespace qq
